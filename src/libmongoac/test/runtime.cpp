#include <mongoac/runtime.h>

#include <catch2/catch_test_macros.hpp>

TEST_CASE("make_progress", "[mongoac][runtime]")
{
   SECTION("null")
   {
      mongoac_runtime_make_progress(nullptr);
      SUCCEED();
   }
}

TEST_CASE("make_progress_with_timeout", "[mongoac][runtime]")
{
   SECTION("null")
   {
      mongoac_runtime_make_progress_with_timeout(nullptr, 0, nullptr);
      SUCCEED();
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
      mongoac_runtime_wait_with_timeout(nullptr, 0, nullptr);
      SUCCEED();
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

TEST_CASE("block_on_any", "[mongoac][runtime]")
{
   SECTION("null")
   {
      CHECK(mongoac_runtime_block_on_any(nullptr, nullptr, 0, nullptr) == nullptr);
   }
}

TEST_CASE("block_on", "[mongoac][runtime]")
{
   SECTION("null")
   {
      mongoac_runtime_block_on(nullptr, nullptr, nullptr);
      SUCCEED();
   }
}

TEST_CASE("block_on_all", "[mongoac][runtime]")
{
   SECTION("null")
   {
      mongoac_runtime_block_on_all(nullptr, nullptr, 0, nullptr);
      SUCCEED();
   }
}

TEST_CASE("block_on_with_timeout", "[mongoac][runtime]")
{
   SECTION("null")
   {
      mongoac_runtime_block_on_with_timeout(nullptr, nullptr, 0, nullptr);
      SUCCEED();
   }
}

TEST_CASE("block_on_any_with_timeout", "[mongoac][runtime]")
{
   SECTION("null")
   {
      CHECK(mongoac_runtime_block_on_any_with_timeout(nullptr, nullptr, 0, 0, nullptr) == nullptr);
   }
}

TEST_CASE("block_on_all_with_timeout", "[mongoac][runtime]")
{
   SECTION("null")
   {
      mongoac_runtime_block_on_all_with_timeout(nullptr, nullptr, 0, 0, nullptr);
      SUCCEED();
   }
}

TEST_CASE("request_stop", "[mongoac][runtime]")
{
   SECTION("null")
   {
      mongoac_runtime_request_stop(nullptr);
      SUCCEED();
   }
}

TEST_CASE("stop_requested", "[mongoac][runtime]")
{
   SECTION("null")
   {
      CHECK(!mongoac_runtime_stop_requested(nullptr));
   }
}
