#include "lua.h"
#include "lapi.h"
#include "lobject.h"
#include "lstate.h"
#include "lute/time.h"

// Wrapper to expose luteopen_time as a C function
extern "C" int lutec_opentime(lua_State *L)
{
    return luteopen_time(L);
}
