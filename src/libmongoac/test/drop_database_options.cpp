#include <mongoac/drop_database_options.h>

#include <catch2/catch_test_macros.hpp>
#include <mongoac/write_concern.h>
#include <test_util/owning_ptr.hh>

using mongoac::test_util::make_owning_ptr;

TEST_CASE("new", "[mongoac][drop_database_options]")
{
   SECTION("default")
   {
      auto const opts = make_owning_ptr(mongoac_drop_database_options_new(), &mongoac_drop_database_options_destroy);
      REQUIRE(opts != nullptr);
   }
}

TEST_CASE("destroy", "[mongoac][drop_database_options]")
{
   SECTION("null")
   {
      mongoac_drop_database_options_destroy(nullptr);
      SUCCEED();
   }
}

TEST_CASE("set_write_concern", "[mongoac][drop_database_options]")
{
   auto const opts = make_owning_ptr(mongoac_drop_database_options_new(), &mongoac_drop_database_options_destroy);

   SECTION("null handle")
   {
      mongoac_drop_database_options_set_write_concern(nullptr, nullptr);
      SUCCEED();
   }

   SECTION("null write concern clears")
   {
      auto const wc = make_owning_ptr(mongoac_write_concern_new(), &mongoac_write_concern_destroy);
      mongoac_drop_database_options_set_write_concern(opts, wc);
      mongoac_drop_database_options_set_write_concern(opts, nullptr);
      SUCCEED();
   }

   SECTION("valid")
   {
      auto const wc = make_owning_ptr(mongoac_write_concern_new(), &mongoac_write_concern_destroy);
      mongoac_drop_database_options_set_write_concern(opts, wc);
      SUCCEED();
   }
}
