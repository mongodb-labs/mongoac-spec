#include <mongoac/runtime.h>

#include <catch2/catch_test_macros.hpp>
#include <mongoac/client.h>

TEST_CASE("make_progress", "[mongoac][runtime]")
{
   SECTION("returns false on null")
   {
      CHECK(!mongoac_runtime_make_progress(nullptr));
   }
}

TEST_CASE("make_progress_with_timeout", "[mongoac][runtime]")
{
   SECTION("returns false on null")
   {
      CHECK(!mongoac_runtime_make_progress_with_timeout(nullptr, 1000));
   }
}

TEST_CASE("runtime destroy", "[mongoac][runtime]")
{
   SECTION("null is safe")
   {
      mongoac_runtime_destroy(nullptr);
      SUCCEED();
   }
}
