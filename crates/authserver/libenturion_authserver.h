#ifndef libenturion_authserver_h
#define libenturion_authserver_h

#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

enum AuthCommand
#ifdef __cplusplus
  : uint8_t
#endif // __cplusplus
 {
  AUTH_LOGON_CHALLENGE = 0,
  AUTH_LOGON_PROOF = 1,
  AUTH_RECONNECT_CHALLENGE = 2,
  AUTH_RECONNECT_PROOF = 3,
  REALM_LIST = 16,
  XFER_INITIATE = 48,
  XFER_DATA = 49,
  XFER_ACCEPT = 50,
  XFER_RESUME = 51,
  XFER_CANCEL = 52,
};
#ifndef __cplusplus
typedef uint8_t AuthCommand;
#endif // __cplusplus

enum AuthResult
#ifdef __cplusplus
  : uint8_t
#endif // __cplusplus
 {
  WOW_SUCCESS = 0,
  WOW_FAIL_BANNED = 3,
  WOW_FAIL_UNKNOWN_ACCOUNT = 4,
  WOW_FAIL_INCORRECT_PASSWORD = 5,
  WOW_FAIL_ALREADY_ONLINE = 6,
  WOW_FAIL_NO_TIME = 7,
  WOW_FAIL_DB_BUSY = 8,
  WOW_FAIL_VERSION_INVALID = 9,
  WOW_FAIL_VERSION_UPDATE = 10,
  WOW_FAIL_INVALID_SERVER = 11,
  WOW_FAIL_SUSPENDED = 12,
  WOW_FAIL_FAIL_NOACCESS = 13,
  WOW_SUCCESS_SURVEY = 14,
  WOW_FAIL_PARENTCONTROL = 15,
  WOW_FAIL_LOCKED_ENFORCED = 16,
  WOW_FAIL_TRIAL_ENDED = 17,
  WOW_FAIL_USE_BATTLENET = 18,
  WOW_FAIL_ANTI_INDULGENCE = 19,
  WOW_FAIL_EXPIRED = 20,
  WOW_FAIL_NO_GAME_ACCOUNT = 21,
  WOW_FAIL_CHARGEBACK = 22,
  WOW_FAIL_INTERNET_GAME_ROOM_WITHOUT_BNET = 23,
  WOW_FAIL_GAME_ACCOUNT_LOCKED = 24,
  WOW_FAIL_UNLOCKABLE_LOCK = 25,
  WOW_FAIL_CONVERSION_REQUIRED = 32,
  WOW_FAIL_DISCONNECTED = 255,
};
#ifndef __cplusplus
typedef uint8_t AuthResult;
#endif // __cplusplus

typedef void (*TickCallback)(void);

typedef struct LogonChallengeErrorResponse {
  AuthCommand command;
  uint8_t padding;
  AuthResult auth_result;
} LogonChallengeErrorResponse;

#ifdef __cplusplus
extern "C" {
#endif // __cplusplus

extern void AbortHandler(void);

extern const Config *ConfigGetInstance(void);

void AuthServerRsInit(void);

void AuthServerRsMain(TickCallback tick_callback);

extern void AuthSession_Free(void *auth_session);

extern void *AuthSession_New(void *rs_auth_session);

extern void AuthSession_Start(const void *auth_session);

extern void AuthSession_Update(const void *auth_session);

extern void AuthSession_WriteIntoBuffer(const void *auth_session, const void *data, uintptr_t size);

const char *AuthSession_GetRemoteIpAddress(const void *this_);

uint16_t AuthSession_GetRemotePort(const void *this_);

void AuthSession_WritePacket(const void *this_, const uint8_t *data, uintptr_t size);

void AuthSession_Disconnect(const void *this_);

void AuthSession_Shutdown(const void *this_);

struct LogonChallengeErrorResponse LogonChallengeErrorResponse_New(AuthCommand command,
                                                                   uint8_t padding,
                                                                   AuthResult auth_result);

void LogonChallengeErrorResponse_Send(struct LogonChallengeErrorResponse self, const void *session);

#ifdef __cplusplus
} // extern "C"
#endif // __cplusplus

#endif /* libenturion_authserver_h */
