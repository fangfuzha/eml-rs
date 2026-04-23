#include <stdio.h>
#include <stdlib.h>

#include "../../include/eml_rs.h"

static void check_status(int32_t code, const char* api_name) {
    if (code == EML_FFI_OK) {
        return;
    }
    fprintf(stderr, "%s failed with status=%d\n", api_name, (int)code);
    exit(1);
}

int main(void) {
    double real_out = 0.0;
    check_status(eml_rs_eval_real(0.7, 2.5, &real_out), "eml_rs_eval_real");
    printf("real eml(0.7, 2.5) = %.12f\n", real_out);

    EmlComplexC complex_out = {0.0, 0.0};
    check_status(
        eml_rs_eval_complex(0.3, 0.2, 1.4, -0.5, &complex_out),
        "eml_rs_eval_complex"
    );
    printf(
        "complex eml(0.3+0.2i, 1.4-0.5i) = %.12f + %.12fi\n",
        complex_out.re,
        complex_out.im
    );

    EmlEvalPolicyC policy = {
        .log_branch = 1,       /* corrected-real */
        .special_values = 1,   /* propagate */
        .near_real_epsilon = 1e-12
    };
    check_status(
        eml_rs_eval_complex_with_policy(-0.2, 0.0, -2.0, 0.0, policy, &complex_out),
        "eml_rs_eval_complex_with_policy"
    );
    printf(
        "policy eml(-0.2+0i, -2+0i) = %.12f + %.12fi\n",
        complex_out.re,
        complex_out.im
    );

    return 0;
}
