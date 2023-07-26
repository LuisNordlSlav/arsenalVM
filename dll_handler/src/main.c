#include <stdio.h>

#ifdef _WIN32

// Windows specific headers
#include <Windows.h>

void* LoadDLL(const char* name) {
    return (void*)LoadLibraryA(name);
}

void DeleteDLL(void* dll) {
    if (dll) {
        FreeLibrary((HMODULE)dll);
    }
}

void* LocateSymbol(void* dll, const char* sym_name) {
    if (!dll) {
        return NULL;
    }

    return (void*)GetProcAddress((HMODULE)dll, sym_name);
}

#else

// Linux specific headers
#include <dlfcn.h>

void* LoadDLL(const char* name) {
    return dlopen(name, RTLD_LAZY | RTLD_GLOBAL);
}

void DeleteDLL(void* dll) {
    if (dll) {
        dlclose(dll);
    }
}

void* LocateSymbol(void* dll, const char* sym_name) {
    if (!dll) {
        return NULL;
    }

    return dlsym(dll, sym_name);
}

#endif
