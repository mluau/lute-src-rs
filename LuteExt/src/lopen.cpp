#include "lua.h"

#include "lute/time.h"
#include "lute/task.h"
#include "lute/crypto.h"
#include "lute/fs.h"
#include "lute/luau.h"
#include "lute/net.h"
#include "lute/process.h"
#include "lute/vm.h"
#include "lute/system.h"
#include "lute/runtime.h"
#include "lute/clicommands.h"
#include "uv.h"

struct lua_State_wrapper
{
    lua_State *parent;
    lua_State *L = nullptr;         // Initialize to nullptr explicitly
    lua_State *DC = nullptr;        // Data copy VM, initialize to nullptr explicitly
    void *runtime_to_set = nullptr; // Pointer to the runtime to set, if needed
};

struct lutec_setupState
{
    void (*setup_lua_state)(lua_State_wrapper *L);
};

// Populates function pointers in the given lute_setupState.
typedef void (*lutec_setupState_init)(lutec_setupState *config);

static lutec_setupState *lutec_setup = nullptr;

extern "C" int lutec_set_runtimeinitter(lutec_setupState_init config_init)
{
    if (lutec_setup)
    {
        return 0; // No-op
    }
    if (!config_init)
    {
        return 1; // Invalid setup state
    }

    lutec_setupState *lute_setup_ptr = new lutec_setupState();

    config_init(lute_setup_ptr); // SAFETY: lute_setup is allocated on the heap

    if (lute_setup_ptr->setup_lua_state == nullptr)
    {
        delete lute_setup_ptr; // Clean up if the setup state is invalid
        lute_setup_ptr = nullptr;
        return 2; // Invalid setup state
    }
    lutec_setup = lute_setup_ptr;

    return 0; // Successfully set up the runtime
}

/*
static void luteopen_lib(lua_State *L, const char *name)
{
    std::unordered_map<const char *, lua_CFunction> libs = {{
        {"@lute/crypto", luteopen_crypto},
        {"@lute/fs", luteopen_fs},
        {"@lute/luau", luteopen_luau},
        {"@lute/net", luteopen_net},
        {"@lute/process", luteopen_process},
        {"@lute/task", luteopen_task},
        {"@lute/vm", luteopen_vm},
        {"@lute/system", luteopen_system},
        {"@lute/time", luteopen_time},
    }};
}*/

#ifndef LUTE_DISABLE_CRYPTO
extern "C" int lutec_opencrypto(lua_State *L)
{
    return luteopen_crypto(L);
}
#endif

extern "C" int lutec_openfs(lua_State *L)
{
    return luteopen_fs(L);
}

extern "C" int lutec_openluau(lua_State *L)
{
    return luteopen_luau(L);
}

#ifndef LUTE_DISABLE_NET
extern "C" int lutec_opennet(lua_State *L)
{
    return luteopen_net(L);
}
#endif

extern "C" int lutec_openprocess(lua_State *L)
{
    return luteopen_process(L);
}

extern "C" int lutec_opentask(lua_State *L)
{
    return luteopen_task(L);
}

extern "C" int lutec_openvm(lua_State *L)
{
    return luteopen_vm(L);
}

extern "C" int lutec_opensystem(lua_State *L)
{
    return luteopen_vm(L);
}

extern "C" int lutec_opentime(lua_State *L)
{
    return luteopen_time(L);
}

// Needed for Lute to link
//
// This always returns NotFound as CLI Filesystem is not supported in embedding
// contexts
CliModuleResult getCliModule(std::string_view path)
{
    return {CliModuleType::NotFound};
}

// Needed for Lute.VM
lua_State *setupState(lua_State *parent, Runtime &runtime, void (*doBeforeSandbox)(lua_State *))
{
    // Make lua_State_wrapper to hold the Lua state and data copy VM that setup_lua_state will fill in
    lua_State_wrapper *lua_state_wrapper = new lua_State_wrapper();
    lua_state_wrapper->parent = parent;
    lua_state_wrapper->L = nullptr;               // Initialize to nullptr explicitly
    lua_state_wrapper->DC = nullptr;              // Data copy VM, initialize to nullptr explicitly as the setup_lua_state will set it
    lua_state_wrapper->runtime_to_set = &runtime; // Set the runtime to set

    lutec_setup->setup_lua_state(lua_state_wrapper);

    lua_State *L = std::move(lua_state_wrapper->L);
    lua_State *DC = std::move(lua_state_wrapper->DC);

    delete lua_state_wrapper; // We can delete the wrapper now

    if (L == nullptr || DC == nullptr)
    {
        return nullptr; // Invalid setup state
    }

    // Ensure setup_lua_state has set the thread data on L before returning
    Runtime *existingRuntime = static_cast<Runtime *>(lua_getthreaddata(L));
    if (!existingRuntime)
        return nullptr; // Thread data already set, cannot set runtime

    runtime.dataCopy.reset(DC);
    runtime.globalState.reset(L);
    L = runtime.globalState.get();
    runtime.GL = L;

    return L;
}

// Wrapper to return whether or not Lute runtime is loaded into a lua state
extern "C" int lutec_isruntimeloaded(lua_State *L)
{
    Runtime *runtime = static_cast<Runtime *>(lua_getthreaddata(L));
    if (runtime)
    {
        return 1;
    }
    return 0;
}

// Wrapper to load the Lute runtime into the Lua state returning the created state
extern "C" void lutec_setup_runtime(lua_State *L)
{
    Runtime *runtime = new Runtime();

    runtime->dataCopy.reset(luaL_newstate());

    runtime->globalState.reset(L);
    runtime->GL = L;

    lua_setthreaddata(L, runtime);
    return;
}

// Wrapper to destroy the Lute runtime inside the lua_State
extern "C" int lutec_destroy_runtime(lua_State *L)
{
    Runtime *runtime = static_cast<Runtime *>(lua_getthreaddata(L));

    if (runtime)
    {
        runtime->stop.store(true);

        if (runtime->globalState)
        {
            runtime->globalState.release();
            runtime->GL = nullptr;
        }
        if (runtime->dataCopy)
        {
            lua_State *DC = runtime->dataCopy.get();
            lua_close(DC);
            runtime->dataCopy.release();
        }

        lua_setthreaddata(L, nullptr);
        delete runtime;

        return 0;
    }
    else
    {
        return 1;
    }
}

extern "C" const int LUTE_STATE_MISSING_ERROR = 0;
extern "C" const int LUTE_STATE_ERROR = 1;
extern "C" const int LUTE_STATE_SUCCESS = 2;
extern "C" const int LUTE_STATE_EMPTY = 3;
extern "C" const int LUTE_STATE_UNSUPPORTED_OP = 4;

extern "C" struct RunOnceResult
{
    int op = LUTE_STATE_UNSUPPORTED_OP; // Default to unsupported operation
    lua_State *state = nullptr;         // The lua_State that was run, if applicable
};

// Wrapper to run one iteration of the Lute scheduler
RunOnceResult lutec_run_once_internal(Runtime *runtime)
{
    auto step = runtime->runOnce();
    if (auto err = Luau::get_if<StepErr>(&step))
    {
        if (err->L == nullptr)
        {
            return RunOnceResult{
                .op = LUTE_STATE_MISSING_ERROR,
                .state = nullptr};
        }

        return RunOnceResult{
            .op = LUTE_STATE_ERROR,
            .state = err->L};
    }
    else if (auto success = Luau::get_if<StepSuccess>(&step))
    {
        return RunOnceResult{
            .op = LUTE_STATE_SUCCESS,
            .state = success->L};
    }
    else if (Luau::get_if<StepEmpty>(&step))
    {
        return RunOnceResult{
            .op = LUTE_STATE_EMPTY,
        };
    }
    else
    {
        return RunOnceResult{
            .op = LUTE_STATE_UNSUPPORTED_OP,
        };
    }
}

LUALIB_API RunOnceResult lutec_run_once(lua_State *L)
{
    Runtime *runtime = static_cast<Runtime *>(lua_getthreaddata(L));

    if (runtime == nullptr)
    {
        return RunOnceResult{
            .op = LUTE_STATE_MISSING_ERROR,
        };
    }

    return lutec_run_once_internal(runtime);
}

LUALIB_API int lutec_run_once_lua(lua_State *L)
{
    Runtime *runtime = static_cast<Runtime *>(lua_getthreaddata(L));

    if (runtime == nullptr)
    {
        luaL_errorL(L, "No runtime state found");
    }

    auto ror = lutec_run_once_internal(runtime);

    switch (ror.op)
    {
    case LUTE_STATE_MISSING_ERROR:
        luaL_errorL(L, "Missing lua state");
    case LUTE_STATE_ERROR:
        lua_xmove(ror.state, L, 1);
        lua_error(L);
    case LUTE_STATE_SUCCESS:
        lua_pushinteger(L, ror.op);
        lua_pushthread(ror.state);
        lua_xmove(ror.state, L, 1);
        return 2;
    case LUTE_STATE_EMPTY:
        lua_pushinteger(L, ror.op);
        lua_pushnil(L);
        return 2;
    case LUTE_STATE_UNSUPPORTED_OP:
        luaL_errorL(L, "Unsupported response from scheduler");
    }
}

LUALIB_API int lutec_has_work(lua_State *L)
{
    Runtime *runtime = static_cast<Runtime *>(lua_getthreaddata(L));

    if (runtime == nullptr)
    {
        return 0;
    }

    bool result = runtime->hasWork();
    if (result == true)
    {
        return 1; // There is work to do
    }
    else
    {
        return 0; // No work to do
    }
}

/*
bool Runtime::hasWork()
{
    return hasContinuations() || hasThreads() || activeTokens.load() != 0;
}
*/

LUALIB_API int lutec_has_continuation(lua_State *L)
{
    Runtime *runtime = static_cast<Runtime *>(lua_getthreaddata(L));

    if (runtime == nullptr)
    {
        return 0;
    }

    bool result = runtime->hasContinuations();
    if (result == true)
    {
        return 1; // There is work to do
    }
    else
    {
        return 0; // No work to do
    }
}

LUALIB_API int lutec_has_threads(lua_State *L)
{
    Runtime *runtime = static_cast<Runtime *>(lua_getthreaddata(L));

    if (runtime == nullptr)
    {
        return 0;
    }

    bool result = runtime->hasThreads();
    if (result == true)
    {
        return 1; // There is work to do
    }
    else
    {
        return 0; // No work to do
    }
}
