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

// Needed for Lute.VM to link right now
lua_State *setupState(Runtime &runtime)
{
    // Separate VM for data copies
    runtime.dataCopy.reset(luaL_newstate());

    runtime.globalState.reset(luaL_newstate());

    lua_State *L = runtime.globalState.get();

    runtime.GL = L;

    lua_setthreaddata(L, &runtime);

    // register the builtin tables
    luaL_openlibs(L);

    luaL_sandbox(L);

    return L;
}

// Wrapper to load the Lute runtime into the Lua state returning the created state
extern "C" void lutec_setup_runtime(lua_State *L, lua_State *DC)
{
    Runtime *runtime = new Runtime();

    runtime->dataCopy.reset(DC);

    runtime->globalState.reset(L);
    runtime->GL = L;

    lua_setthreaddata(L, runtime);
    return;
}

// Wrapper to destroy the Lute runtime inside the lua_State
extern "C" int lutec_destroy_runtime(lua_State *L)
{
    Runtime *runtime = static_cast<Runtime *>(lua_getthreaddata(L));

    printf("Destroying Lute runtime\n");

    if (runtime)
    {
        printf("Lute runtime found\n");

        if (runtime->globalState)
        {
            printf("Lute runtime global state found\n");
            runtime->globalState.release();
            runtime->GL = nullptr;
        }
        if (runtime->dataCopy)
        {
            printf("Lute runtime data copy found\n");
            runtime->dataCopy.release();
            printf("Lute runtime data copy released\n");
        }

        printf("Lute runtime data copy deleted\n");
        lua_setthreaddata(L, nullptr);
        delete runtime;
        printf("Lute runtime global state deleted\n");
        return 0;
    }
    else
    {
        return 1;
    }
}

// Wrapper to run one iteration of the Lute scheduler
extern "C" int lutec_run_once(Runtime *runtime)
{
    // Complete all C++ continuations
    std::vector<std::function<void()>> copy;

    {
        std::unique_lock lock(runtime->continuationMutex);
        copy = std::move(runtime->continuations);
        runtime->continuations.clear();
    }

    for (auto &&continuation : copy)
        continuation();

    if (runtime->runningThreads.empty())
        return 0;

    // Run the next thread
    auto next = std::move(runtime->runningThreads.front());
    runtime->runningThreads.erase(runtime->runningThreads.begin());

    next.ref->push(runtime->GL);
    lua_State *L = lua_tothread(runtime->GL, -1);

    if (L == nullptr)
    {
        luaL_error(runtime->GL, "Cannot resume a non-thread reference");
        return -1;
    }

    // We still have 'next' on stack to hold on to thread we are about to run
    lua_pop(runtime->GL, 1);

    int status = LUA_OK;

    if (!next.success)
        status = lua_resumeerror(L, nullptr);
    else
        status = lua_resume(L, nullptr, next.argumentCount);

    if (status == LUA_YIELD)
    {
        // Yielding, continue to next iteration
        return 0;
    }

    if (status != LUA_OK)
    {
        std::string error;

        if (const char *str = lua_tostring(L, -1))
            error = str;

        error += "\nstacktrace:\n";
        error += lua_debugtrace(L);

        luaL_error(runtime->GL, "%s", error.c_str());
        return false;
    }

    if (next.cont)
        next.cont();

    return 0;
}