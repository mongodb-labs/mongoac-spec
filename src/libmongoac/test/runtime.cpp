#include <mongoac/runtime.h>

#include <catch2/catch_test_macros.hpp>

TEST_CASE("make_progress", "[mongoac][runtime]")
{
   SECTION("null")
   {
      CHECK(!mongoac_runtime_make_progress(nullptr));
   }
}

TEST_CASE("make_progress_with_timeout", "[mongoac][runtime]")
{
   SECTION("null")
   {
      CHECK(!mongoac_runtime_make_progress_with_timeout(nullptr, 0));
   }
}

TEST_CASE("wait", "[mongoac][runtime]")
{
   SECTION("null")
   {
      mongoac_runtime_wait(nullptr);
      SUCCEED();
   }
}

TEST_CASE("wait_with_timeout", "[mongoac][runtime]")
{
   SECTION("null")
   {
      CHECK(!mongoac_runtime_wait_with_timeout(nullptr, 0));
   }
}

TEST_CASE("runtime destroy", "[mongoac][runtime]")
{
   SECTION("null")
   {
      mongoac_runtime_destroy(nullptr);
      SUCCEED();
   }
}
