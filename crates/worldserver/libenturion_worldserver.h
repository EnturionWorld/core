#ifndef libenturion_worldserver_h
#define libenturion_worldserver_h

#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

typedef void (*TickCallback)(void);

#ifdef __cplusplus
extern "C" {
#endif // __cplusplus

extern void AbortHandler(void);

extern int32_t World_IsStopped(void);

void WorldServerRsInit(void);

void WorldServerRsMain(TickCallback tick_callback);

#ifdef __cplusplus
} // extern "C"
#endif // __cplusplus

#endif /* libenturion_worldserver_h */
