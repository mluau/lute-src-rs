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
#include "uv.h"

typedef struct
{
    lua_State *L;
} lua_State_wrapper;

struct lutec_setupState
{
    // Returns a new lua_State that has been setup by the caller
    void (*setup_lua_state)(lua_State_wrapper *L);
};

// Populates function pointers in the given lute_setupState.
typedef void (*lutec_setupState_init)(lutec_setupState *config);

static lutec_setupState *lutec_setup = nullptr;

extern "C" int lutec_set_runtimeinitter(lutec_setupState_init config_init)
{
    printf("lutec_set_runtimeinitter called\n");
    if (lutec_setup)
    {
        printf("ERROR: lutec_set_runtimeinitter called after setup\n");
        return -1; // Cannot modify the setup state after it has been set
    }
    if (!config_init)
    {
        printf("ERROR: lutec_set_runtimeinitter called with null config_init\n");
        return -2; // Invalid setup state
    }

    printf("Creating new lutec_setupState\n");
    lutec_setupState *lute_setup_ptr = new lutec_setupState();

    config_init(lute_setup_ptr); // SAFETY: lute_setup is allocated on the heap

    if (lute_setup_ptr->setup_lua_state == nullptr)
    {
        printf("ERROR: lutec_set_runtimeinitter called with null setup_lua_state\n");
        delete lute_setup_ptr;
        lute_setup_ptr = nullptr;
        return -3; // Invalid setup state
    }
    lutec_setup = lute_setup_ptr;
    printf("lutec_set_runtimeinitter callback done\n");
    // Test by calling the setup_lua_state function
    lua_State_wrapper *lua_state_wrapper = new lua_State_wrapper();

    lutec_setup->setup_lua_state(lua_state_wrapper);

    lua_State *L = lua_state_wrapper->L;
    if (L == nullptr)
    {
        printf("ERROR: setup_lua_state returned null lua_State\n");
        delete lute_setup_ptr;
        lutec_setup = nullptr;
        return -5; // Invalid setup state
    }

    printf("lutec_set_runtimeinitter done\n");

    return 0;
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

extern "C" int lutec_opencrypto(lua_State *L)
{
    return luteopen_crypto(L);
}

extern "C" int lutec_openfs(lua_State *L)
{
    return luteopen_fs(L);
}

extern "C" int lutec_openluau(lua_State *L)
{
    return luteopen_luau(L);
}

extern "C" int lutec_opennet(lua_State *L)
{
    return luteopen_net(L);
}

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

// Needed for Lute.VM
lua_State *setupState(Runtime &runtime)
{
    printf("setupState called\n");

    // Make data copy VM
    lua_State_wrapper *lua_state_wrapper = new lua_State_wrapper();

    lutec_setup->setup_lua_state(lua_state_wrapper);

    lua_State *DC = std::move(lua_state_wrapper->L);
    if (DC == nullptr)
    {
        printf("ERROR: setup_lua_state returned null DC\n");
        delete lua_state_wrapper;
        return nullptr; // Invalid setup state
    }

    // Separate VM for data copies
    runtime.dataCopy.reset(DC);

    // Drop lua_state_wrapper
    delete lua_state_wrapper;

    // Create the main VM
    lua_State_wrapper *lua_state_wrapper_main = new lua_State_wrapper();
    lutec_setup->setup_lua_state(lua_state_wrapper_main);
    lua_State *L = std::move(lua_state_wrapper_main->L);
    if (L == nullptr)
    {
        printf("ERROR: setup_lua_state returned null L\n");
        delete lua_state_wrapper_main;
        return nullptr; // Invalid setup state
    }

    runtime.globalState.reset(L);

    delete lua_state_wrapper_main;

    L = runtime.globalState.get();

    runtime.GL = L;

    lua_setthreaddata(L, &runtime);

    // register the builtin tables
    luaL_openlibs(L);

    luaL_sandbox(L);

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

    lua_gc(L, LUA_GCCOLLECT, 2);
    lua_gc(L, LUA_GCCOLLECT, 2);
    lua_gc(L, LUA_GCCOLLECT, 0);
    lua_gc(L, LUA_GCCOLLECT, 0);
    lua_gc(L, LUA_GCCOLLECT, 0);

    printf("Destroying Lute runtime\n");

    if (runtime)
    {
        printf("Lute runtime found\n");

        runtime->stop.store(true);

        if (runtime->globalState)
        {
            printf("Lute runtime global state found\n");
            runtime->globalState.release();
            runtime->GL = nullptr;
        }
        if (runtime->dataCopy)
        {
            printf("Lute runtime data copy found\n");
            lua_State *DC = runtime->dataCopy.get();
            lua_close(DC);
            runtime->dataCopy.release();
            printf("Lute runtime data copy released\n");
        }

        printf("Lute runtime data copy deleted\n");

        lua_setthreaddata(L, nullptr);
        delete runtime;
        printf("Lute runtime global state deleted\n");

        // Run 2 gc cycles to clean up the memory
        lua_gc(L, LUA_GCCOLLECT, 2);
        lua_gc(L, LUA_GCCOLLECT, 2);
        lua_gc(L, LUA_GCCOLLECT, 0);
        lua_gc(L, LUA_GCCOLLECT, 0);
        lua_gc(L, LUA_GCCOLLECT, 0);

        return 0;
    }
    else
    {
        return 1;
    }
}

// Wrapper to run one iteration of the Lute scheduler
int lutec_run_once_internal(Runtime *runtime)
{
    uv_run(uv_default_loop(), UV_RUN_DEFAULT);

    // luaL_error(runtime->GL, "Lute scheduler run_once called without a runtime");

    // Complete all C++ continuations
    std::vector<std::function<void()>> copy;

    {
        std::unique_lock lock(runtime->continuationMutex);
        copy = std::move(runtime->continuations);
        runtime->continuations.clear();
    }

    for (auto &&continuation : copy)
    {
        printf("Running continuation\n");
        continuation();
    }

    if (runtime->runningThreads.empty())
    {
        printf("No threads to run\n");
        // Push code 1000 to indicate nothing to run
        lua_pushinteger(runtime->GL, 1000);
        lua_pushnil(runtime->GL);
        return 2;
    }

    // Run the next thread
    auto next = std::move(runtime->runningThreads.front());
    runtime->runningThreads.erase(runtime->runningThreads.begin());

    next.ref->push(runtime->GL);
    lua_State *L = lua_tothread(runtime->GL, -1);

    if (L == nullptr)
    {
        printf("ERROR: Cannot resume a non-thread reference\n");
        luaL_error(runtime->GL, "Cannot resume a non-thread reference");
        return -1;
    }

    // We still have 'next' on stack to hold on to thread we are about to run
    lua_pop(runtime->GL, 1);

    int status = LUA_OK;

    printf("Running thread %p\n", L);

    if (!next.success)
        status = lua_resumeerror(L, nullptr);
    else
        status = lua_resume(L, nullptr, next.argumentCount);

    printf("Thread %p finished with status %d\n", L, status);

    if (status == LUA_YIELD)
    {
        // Yielding, continue to next iteration
        printf("Thread yielded\n");
        lua_pushinteger(L, LUA_YIELD);
        lua_pushthread(L);
        return 2;
    }

    if (status != LUA_OK)
    {
        std::string error;

        if (const char *str = lua_tostring(L, -1))
            error = str;

        error += "\nstacktrace:\n";
        error += lua_debugtrace(L);

        printf("ERROR: %s\n", error.c_str());
        luaL_error(runtime->GL, "%s", error.c_str());
        return -1;
    }

    if (next.cont)
    {
        printf("Running continuation\n");
        next.cont();
    }

    printf("Pushing 3, nil to indicate thread finished\n");
    lua_pushinteger(runtime->GL, 3);
    lua_pushnil(runtime->GL);
    return 2;
}

LUALIB_API int lutec_run_once(lua_State *L)
{
    printf("lutec_run_once called\n");

    Runtime *runtime = static_cast<Runtime *>(lua_getthreaddata(L));

    if (runtime == nullptr)
    {
        printf("ERROR: Cannot run Lute scheduler without a runtime\n");
        luaL_error(L, "Cannot run Lute scheduler without a runtime");
        return -1;
    }

    return lutec_run_once_internal(runtime);
}
