#ifndef TRANSCRUST_H
#define TRANSCRUST_H
#include <stddef.h>
#include <stdint.h>
#ifdef __cplusplus
extern "C" {
#endif
typedef struct TcEngine TcEngine;
/* All strings are UTF-8, NUL terminated. JSON responses must be tc_free'd.
 * Call blocking operations on a worker. Never destroy a handle while a call
 * is active. Per-handle operations serialize; concurrent calls return busy.
 * tc_cancel is concurrent-safe; discard results whose cancelled flag is true.
 * The caller owns UI/session/focus identity and all text delivery. */
char *tc_text(const char *request_json);
TcEngine *tc_create(const char *config_json, char **error_json);
char *tc_process(TcEngine *, const char *request_json);
char *tc_transcribe(TcEngine *, const float *mono_pcm, size_t samples, uint32_t rate);
void tc_cancel(TcEngine *);
void tc_destroy(TcEngine *);
void tc_free(char *response_json);
#ifdef __cplusplus
}
#endif
#endif
