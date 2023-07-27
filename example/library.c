#include <stdio.h>

void test_a() {
    printf("Hello, world!\n");
}

int test_b(int a, int b) {
    printf("%d + %d = %d\n", a, b, a + b);
    return a + b;
}

void test_c(void func()) {
    func();
}

void test_d(void func(int)) {
    func(5);
}
