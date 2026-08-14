#include <mongoac/runtime.h>

//

#include <catch2/catch_test_macros.hpp>
#include <mongoac/client.h>
#include <test_util/string.hh>
using mongoac::test_util::to_mongoac;

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

TEST_CASE("make_progress_for", "[mongoac][runtime]")
{
   SECTION("null")
   {
      mongoac_runtime_make_progress_for(nullptr, 0);
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

TEST_CASE("lifetime", "[mongoac][runtime]")
{
   auto const client = mongoac_client_new(to_mongoac("mongodb://localhost:27017"), nullptr);
   REQUIRE(client != nullptr);

   auto const runtime = mongoac_client_get_runtime(client);
   REQUIRE(runtime != nullptr);

   mongoac_client_destroy(client); // RuntimeT may outlive ClientT.

   mongoac_runtime_destroy(runtime);
}
