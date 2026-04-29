#include <sys/mman.h>
#include <sys/stat.h>
#include <fcntl.h>
#include <unistd.h>
#include <stdio.h>
#include <stdlib.h>
#include <errno.h>

void* map_graph(const char* path, size_t size) {
    printf("C Helper: Mapping %s, size %zu\n", path, size);
    int fd = open(path, O_RDONLY);
    if (fd < 0) {
        perror("C Helper Error: open failed");
        return NULL;
    }
    void* ptr = mmap(NULL, size, PROT_READ, MAP_SHARED, fd, 0);
    close(fd);
    if (ptr == MAP_FAILED) {
        perror("C Helper Error: mmap failed");
        return NULL;
    }
    return ptr;
}

void unmap_graph(void* ptr, size_t size) {
    munmap(ptr, size);
}

unsigned long long read_u64(void* ptr, size_t offset) {
    if (!ptr) return 0;
    return *(unsigned long long*)((char*)ptr + offset);
}

unsigned short read_u16(void* ptr, size_t offset) {
    if (!ptr) return 0;
    return *(unsigned short*)((char*)ptr + offset);
}

float read_f32(void* ptr, size_t offset) {
    if (!ptr) return 0.0f;
    return *(float*)((char*)ptr + offset);
}

unsigned int read_u32(void* ptr, size_t offset) {
    if (!ptr) return 0;
    return *(unsigned int*)((char*)ptr + offset);
}
