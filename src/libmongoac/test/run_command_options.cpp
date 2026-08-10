#include <mongoac/run_command_options.h>

#include <catch2/catch_test_macros.hpp>
#include <mongoac/read_preference.h>
#include <test_util/owning_ptr.hh>

using mongoac::test_util::make_owning_ptr;

TEST_CASE("new", "[mongoac][run_command_options]")
{
   auto const opts = make_owning_ptr(mongoac_run_command_options_new(), &mongoac_run_command_options_destroy);
   REQUIRE(opts != nullptr);
}

TEST_CASE("destroy", "[mongoac][run_command_options]")
{
   SECTION("null")
   {
      mongoac_run_command_options_destroy(nullptr);
      SUCCEED();
   }
}

TEST_CASE("set_read_preference", "[mongoac][run_command_options]")
{
   auto const opts = make_owning_ptr(mongoac_run_command_options_new(), &mongoac_run_command_options_destroy);
   auto const rp = make_owning_ptr(mongoac_read_preference_new(), &mongoac_read_preference_destroy);

   SECTION("null handle")
   {
      mongoac_run_command_options_set_read_preference(nullptr, rp);
      SUCCEED();
   }

   SECTION("null preference clears")
   {
      mongoac_run_command_options_set_read_preference(opts, nullptr);
      SUCCEED();
   }

   SECTION("valid")
   {
      mongoac_run_command_options_set_read_preference(opts, rp);
      SUCCEED();
   }
}

TEST_CASE("set_server_selector", "[mongoac][run_command_options]")
{
   auto const opts = make_owning_ptr(mongoac_run_command_options_new(), &mongoac_run_command_options_destroy);

   SECTION("null handle")
   {
      mongoac_run_command_options_set_server_selector(nullptr, nullptr);
      SUCCEED();
   }

   SECTION("null selector clears")
   {
      mongoac_run_command_options_set_server_selector(opts, nullptr);
      SUCCEED();
   }
}
