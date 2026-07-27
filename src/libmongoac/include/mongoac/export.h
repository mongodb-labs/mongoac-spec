#ifndef MONGOAC_EXPORT_H
#define MONGOAC_EXPORT_H

// Avoid -Wempty-translation-unit warnings.
#include <stddef.h>

#ifdef MONGOAC_STATIC
#define MONGOAC_API
#elif defined(_WIN32)
#define MONGOAC_API __declspec(dllimport)
#elif defined(__GNUC__) && __GNUC__ >= 4
#define MONGOAC_API __attribute__((visibility("default")))
#else
#define MONGOAC_API
#endif

#endif // MONGOAC_EXPORT_H
