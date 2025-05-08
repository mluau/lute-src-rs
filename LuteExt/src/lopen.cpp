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
extern "C" void lutec_setup_runtime(lua_State *L)
{
    Runtime *runtime = new Runtime();

    // Separate VM for data copies
    lua_State *DC = luaL_newstate();
    if (DC == nullptr)
    {
        luaL_error(L, "Failed to create data copy state");
        return;
    }

    runtime->dataCopy.reset(DC);

    runtime->globalState.reset(L);
    runtime->GL = L;

    lua_setthreaddata(L, runtime);
    return;
}

// Get the data copy state from the runtime
extern "C" lua_State *lutec_get_data_copy(lua_State *L)
{
    Runtime *runtime = static_cast<Runtime *>(lua_getthreaddata(L));
    if (runtime && runtime->dataCopy)
    {
        return runtime->dataCopy.get();
    }

    if (runtime)
    {
        printf("Lute runtime data copy not found\n");
    }
    else
    {
        printf("Lute runtime not found\n");
    }

    return nullptr;
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
            lua_State *DC = runtime->dataCopy.get();
            printf("Closed dataCopy on runtime");
            printf("Pointer to dataCopy: %p\n", DC);
            if (DC != nullptr)
            {
                lua_close(DC);
            }
            printf("Lute runtime data copy closed\n");
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