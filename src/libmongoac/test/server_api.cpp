#include <mongoac/server_api.h>

#include <catch2/catch_test_macros.hpp>
#include <test_util/owning_ptr.hh>

using mongoac::test_util::make_owning_ptr;

TEST_CASE("new", "[mongoac][server_api]")
{
   SECTION("default")
   {
      auto const api = make_owning_ptr(mongoac_server_api_new(), &mongoac_server_api_destroy);
      REQUIRE(api != nullptr);
   }
}

TEST_CASE("destroy", "[mongoac][server_api]")
{
   SECTION("null")
   {
      mongoac_server_api_destroy(nullptr);
      SUCCEED();
   }
}

TEST_CASE("set_strict", "[mongoac][server_api]")
{
   auto const api = make_owning_ptr(mongoac_server_api_new(), &mongoac_server_api_destroy);

   SECTION("null")
   {
      mongoac_server_api_set_strict(nullptr, true);
      SUCCEED();
   }

   SECTION("set true")
   {
      mongoac_server_api_set_strict(api, true);
      SUCCEED();
   }

   SECTION("set false")
   {
      mongoac_server_api_set_strict(api, false);
      SUCCEED();
   }
}

TEST_CASE("set_deprecation_errors", "[mongoac][server_api]")
{
   auto const api = make_owning_ptr(mongoac_server_api_new(), &mongoac_server_api_destroy);

   SECTION("null")
   {
      mongoac_server_api_set_deprecation_errors(nullptr, true);
      SUCCEED();
   }

   SECTION("set true")
   {
      mongoac_server_api_set_deprecation_errors(api, true);
      SUCCEED();
   }

   SECTION("set false")
   {
      mongoac_server_api_set_deprecation_errors(api, false);
      SUCCEED();
   }
}
