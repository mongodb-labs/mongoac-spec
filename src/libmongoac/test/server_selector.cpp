#include <mongoac/server_selector.h>

#include <catch2/catch_test_macros.hpp>
#include <mongoac/server_info.h>
#include <test_util/owning_ptr.hh>

using mongoac::test_util::make_owning_ptr;

static bool
always_true(mongoac_server_info_t const *info, void *user_data)
{
   (void)info;
   (void)user_data;
   return true;
}

TEST_CASE("new", "[mongoac][server_selector]")
{
   SECTION("valid callback")
   {
      auto const sc =
         make_owning_ptr(mongoac_server_selector_new(&always_true, nullptr), &mongoac_server_selector_destroy);
      REQUIRE(sc != nullptr);
   }

   SECTION("valid callback with user_data")
   {
      int sentinel = 42;
      auto const sc =
         make_owning_ptr(mongoac_server_selector_new(&always_true, &sentinel), &mongoac_server_selector_destroy);
      REQUIRE(sc != nullptr);
   }

   SECTION("null callback returns null")
   {
      auto *sc = mongoac_server_selector_new(nullptr, nullptr);
      REQUIRE(sc == nullptr);
   }
}

TEST_CASE("destroy", "[mongoac][server_selector]")
{
   SECTION("null")
   {
      mongoac_server_selector_destroy(nullptr);
      SUCCEED();
   }
}
