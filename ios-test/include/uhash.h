#ifndef UHASH_H
#define UHASH_H

#include <stdint.h>
#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

// Opaque hasher handle
typedef struct UHasher UHasher;

// Create a new hasher instance (allocates ~2MB)
UHasher* uhash_new(void);

// Free a hasher instance
void uhash_free(UHasher* hasher);

// Compute hash of input data
// - hasher: pointer from uhash_new()
// - input: pointer to input bytes
// - input_len: length of input
// - output: pointer to 32-byte buffer for result
void uhash_hash(UHasher* hasher, const uint8_t* input, size_t input_len, uint8_t* output);

// Benchmark: compute N hashes and return total microseconds
uint64_t uhash_benchmark(uint32_t iterations);

// Calculate hash rate from benchmark results
double uhash_hashrate(uint32_t iterations, uint64_t microseconds);

#ifdef __cplusplus
}
#endif

#endif // UHASH_H
