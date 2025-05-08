#include "lua.h"

#include "lute/time.h"
#include "lute/task.h"
#include "lute/runtime.h"

// Wrapper to expose luteopen_time as a C function
extern "C" int lutec_opentime(lua_State *L)
{
    return luteopen_time(L);
}

// Wrapper to expose luteopen_time as a C function
extern "C" int lutec_opentask(lua_State *L)
{
    return luteopen_task(L);
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