#ifndef EML_RS_H
#define EML_RS_H

#ifdef __cplusplus
extern "C" {
#endif

#include <stdint.h>

#define EML_FFI_OK 0
#define EML_FFI_EVAL_ERROR 1
#define EML_FFI_NULL_OUT 2

typedef struct EmlComplexC {
    double re;
    double im;
} EmlComplexC;

typedef struct EmlEvalPolicyC {
    uint8_t log_branch;       /* 0 => principal, 1 => corrected-real */
    uint8_t special_values;   /* 0 => strict, 1 => propagate */
    double near_real_epsilon;
} EmlEvalPolicyC;

/* Evaluate real eml(x, y) = exp(x) - ln(y). */
int32_t eml_rs_eval_real(double x, double y, double* out);

/* Evaluate complex eml(x, y). */
int32_t eml_rs_eval_complex(
    double x_re,
    double x_im,
    double y_re,
    double y_im,
    EmlComplexC* out
);

/* Evaluate complex eml(x, y) with explicit policy. */
int32_t eml_rs_eval_complex_with_policy(
    double x_re,
    double x_im,
    double y_re,
    double y_im,
    EmlEvalPolicyC policy,
    EmlComplexC* out
);

#ifdef __cplusplus
}
#endif

#endif /* EML_RS_H */
