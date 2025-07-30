#ifndef TEST_GLOBALS_H
#define TEST_GLOBALS_H
#include <chrono>
#include <functional>
#include <future>
#include <gtest/gtest.h>

template<typename ReturnT, typename FuncT, typename UnitT = std::chrono::milliseconds, typename ...ParameterT>
ReturnT timeout_test(FuncT &&func, const UnitT& timeout, ParameterT... params){
    std::atomic cancel_flag {false};
    std::promise<ReturnT> done;
    std::thread t { [&] {
        try{
            std::invoke(std::forward<FuncT>(func), params...);
            if (!cancel_flag.load()){
                done.set_value();
            }
        } catch (...){
            if (!cancel_flag.load()) {
                done.set_exception(std::current_exception());
            }
        }
    }};

    auto future = done.get_future();
    if (future.wait_for(timeout) == std::future_status::timeout){
        cancel_flag = true;
        FAIL() << "Timeout reached after " << timeout << ".\n";
    }

    t.join();
    return future.get();
}
#endif // TEST_GLOBALS_H
