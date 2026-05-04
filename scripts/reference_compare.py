#!/usr/bin/env python3
"""用 NumPy/PyTorch 执行 portable graph JSON，作为研究对照后端。"""

from __future__ import annotations

import argparse
import json
import math
from pathlib import Path
from typing import Any, Callable


def _load_json(path: Path) -> Any:
    with path.open("r", encoding="utf-8") as handle:
        return json.load(handle)


def _default_samples(arity: int) -> list[list[float]]:
    arity = max(arity, 1)
    return [[0.2 + 0.1 * i for i in range(arity)], [0.7 + 0.05 * i for i in range(arity)]]


def _graph_arity(graph: dict[str, Any]) -> int:
    arity = 0
    for node in graph["nodes"]:
        if node["op"] == "var":
            arity = max(arity, int(node["attrs"]["index"]) + 1)
    return arity


def _eval_graph(graph: dict[str, Any], sample: list[float], backend: Any) -> Any:
    values: list[Any] = []
    for node in graph["nodes"]:
        op = node["op"]
        attrs = node.get("attrs", {})
        inputs = [values[index] for index in node.get("inputs", [])]
        values.append(_eval_node(op, attrs, inputs, sample, backend))
    return values[int(graph["root"])]


def _sigmoid(x: Any, backend: Any) -> Any:
    return 1 / (1 + backend.exp(-x))


def _eval_node(op: str, attrs: dict[str, Any], inputs: list[Any], sample: list[float], backend: Any) -> Any:
    if op == "var":
        return backend.asarray(sample[int(attrs["index"])])
    if op == "int":
        return backend.asarray(float(attrs["value"]))
    if op == "rational":
        return backend.asarray(float(attrs["numerator"]) / float(attrs["denominator"]))
    if op == "const_e":
        return backend.asarray(math.e)
    if op == "const_i":
        return backend.asarray(1j)
    if op == "const_pi":
        return backend.asarray(math.pi)
    if op == "one":
        return backend.asarray(1.0)
    if op == "eml":
        return backend.exp(inputs[0]) - backend.log(inputs[1])
    if op == "neg":
        return -inputs[0]
    if op == "add":
        return inputs[0] + inputs[1]
    if op == "sub":
        return inputs[0] - inputs[1]
    if op == "mul":
        return inputs[0] * inputs[1]
    if op == "div":
        return inputs[0] / inputs[1]
    if op == "pow":
        return backend.exp(backend.log(inputs[0]) * inputs[1])
    unary: dict[str, Callable[[Any], Any]] = {
        "exp": backend.exp,
        "log": backend.log,
        "sin": backend.sin,
        "cos": backend.cos,
        "tan": backend.tan,
        "sinh": backend.sinh,
        "cosh": backend.cosh,
        "tanh": backend.tanh,
        "asin": backend.arcsin,
        "acos": backend.arccos,
        "atan": backend.arctan,
        "sqrt": backend.sqrt,
    }
    if op in unary:
        return unary[op](inputs[0])
    if op == "sigmoid":
        return _sigmoid(inputs[0], backend)
    if op == "softplus" or op == "relu_soft":
        return backend.log(1 + backend.exp(inputs[0]))
    if op == "swish":
        return inputs[0] * _sigmoid(inputs[0], backend)
    if op == "gelu_tanh":
        return 0.5 * inputs[0] * (1 + backend.tanh(math.sqrt(2 / math.pi) * (inputs[0] + 0.044715 * inputs[0] ** 3)))
    if op == "elu":
        beta = 8.0
        gate = _sigmoid(beta * inputs[0], backend)
        return inputs[0] * gate + inputs[1] * (backend.exp(inputs[0]) - 1) * (1 - gate)
    if op == "leaky_relu":
        beta = 8.0
        gate = _sigmoid(beta * inputs[0], backend)
        return inputs[0] * (gate + inputs[1] * (1 - gate))
    if op == "softsign":
        return inputs[0] / (1 + backend.sqrt(inputs[0] * inputs[0] + 0.01))
    if op == "mish":
        return inputs[0] * backend.tanh(backend.log(1 + backend.exp(inputs[0])))
    raise ValueError(f"unsupported op: {op}")


def _as_float(value: Any) -> float:
    if hasattr(value, "detach"):
        value = value.detach().cpu().numpy()
    if hasattr(value, "item"):
        value = value.item()
    return float(value.real if isinstance(value, complex) else value)


def _run_numpy(graph: dict[str, Any], samples: list[list[float]]) -> list[float]:
    import numpy as np

    return [_as_float(_eval_graph(graph, sample, np)) for sample in samples]


def _run_torch(graph: dict[str, Any], samples: list[list[float]]) -> list[float] | None:
    try:
        import torch
    except ImportError:
        return None

    class TorchBackend:
        @staticmethod
        def asarray(value: Any) -> Any:
            return torch.as_tensor(value, dtype=torch.float64)

    for name in [
        "exp",
        "log",
        "sin",
        "cos",
        "tan",
        "sinh",
        "cosh",
        "tanh",
        "sqrt",
        "arcsin",
        "arccos",
        "arctan",
    ]:
        setattr(TorchBackend, name, staticmethod(getattr(torch, name)))
    return [_as_float(_eval_graph(graph, sample, TorchBackend)) for sample in samples]


def main() -> int:
    parser = argparse.ArgumentParser(description="Run NumPy/PyTorch reference evaluation for portable graph JSON.")
    parser.add_argument("--graph", required=True, type=Path, help="portable graph JSON path")
    parser.add_argument("--samples", type=Path, help="optional samples JSON, shape: [[x0, x1], ...]")
    parser.add_argument("--tolerance", type=float, default=1e-8, help="NumPy/Torch comparison tolerance")
    args = parser.parse_args()

    graph = _load_json(args.graph)
    samples = _load_json(args.samples) if args.samples else _default_samples(_graph_arity(graph))
    numpy_values = _run_numpy(graph, samples)
    torch_values = _run_torch(graph, samples)

    result = {
        "graph": str(args.graph),
        "samples": len(samples),
        "numpy": numpy_values,
        "torch": torch_values,
        "torch_available": torch_values is not None,
        "max_abs_numpy_torch": None,
        "passed": True,
    }
    if torch_values is not None:
        errors = [abs(a - b) for a, b in zip(numpy_values, torch_values)]
        result["max_abs_numpy_torch"] = max(errors) if errors else 0.0
        result["passed"] = result["max_abs_numpy_torch"] <= args.tolerance

    print(json.dumps(result, indent=2, ensure_ascii=False))
    return 0 if result["passed"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
