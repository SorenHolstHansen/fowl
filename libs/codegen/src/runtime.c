// Minimal runtime
#include <stdio.h>
#include <stdlib.h>
#include <stdint.h>
#include <string.h>

// ======================
// Printing
// ======================

void rt_print_str(const char *s) {
    fputs(s, stdout);
}

// ======================
// String creation / memory
// ======================

void *rt_malloc(uint64_t size) {
    return malloc(size);
}

void *rt_realloc(void *ptr, uint64_t size) {
    return realloc(ptr, size);
}

void rt_free(void *ptr) {
    free(ptr);
}

// ======================
// Conversions
// ======================

// Convert int -> heap string
char *rt_int_to_str(int64_t x) {
    // worst case: -9223372036854775808 → 20 chars + null
    char temp[32];
    int len = snprintf(temp, sizeof(temp), "%lld", (long long)x);

    char *out = malloc(len + 1);
    if (!out) return NULL;

    memcpy(out, temp, len + 1);
    return out;
}

// Convert double -> heap string
char *rt_double_to_str(double d) {
    char temp[64];
    int len = snprintf(temp, sizeof(temp), "%g", d);

    char *out = malloc(len + 1);
    if (!out) return NULL;

    memcpy(out, temp, len + 1);
    return out;
}

// Convert string -> int
int64_t rt_str_to_int(const char *s) {
    return strtoll(s, NULL, 10);
}

// ======================
// General string helpers
// ======================

// Concatenate two strings, returning newly allocated result
char *rt_concat(const char *a, const char *b) {
    size_t la = strlen(a);
    size_t lb = strlen(b);

    char *out = malloc(la + lb + 1);
    if (!out) return NULL;

    memcpy(out, a, la);
    memcpy(out + la, b, lb + 1); // includes null terminator

    return out;
}

// ======================
// Input
// ======================

char *rt_read_line() {
    size_t size = 128;
    size_t len = 0;
    char *buffer = malloc(size);

    if (!buffer) return NULL;

    int c;
    while ((c = getchar()) != EOF && c != '\n') {
        if (len + 1 >= size) {
            size *= 2;
            char *newbuf = realloc(buffer, size);
            if (!newbuf) { free(buffer); return NULL; }
            buffer = newbuf;
        }
        buffer[len++] = (char)c;
    }

    buffer[len] = '\0';
    return buffer;
}

// ======================
// Exit
// ======================

void rt_exit(int code) {
    exit(code);
}
