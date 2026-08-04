#include <mongoac/database_options.h>

#include <catch2/catch_test_macros.hpp>
#include <mongoac/read_concern.h>
#include <mongoac/read_preference.h>
#include <mongoac/write_concern.h>
#include <test_util/owning_ptr.hh>

using mongoac::test_util::make_owning_ptr;

TEST_CASE("new", "[mongoac][database_options]")
{
   SECTION("default")
   {
      auto const opts = make_owning_ptr(mongoac_database_options_new(), &mongoac_database_options_destroy);
      REQUIRE(opts != nullptr);
   }
}

TEST_CASE("destroy", "[mongoac][database_options]")
{
   SECTION("null")
   {
      mongoac_database_options_destroy(nullptr);
      SUCCEED();
   }
}

TEST_CASE("set_read_concern", "[mongoac][database_options]")
{
   auto const opts = make_owning_ptr(mongoac_database_options_new(), &mongoac_database_options_destroy);

   SECTION("null handle")
   {
      mongoac_database_options_set_read_concern(nullptr, nullptr);
      SUCCEED();
   }

   SECTION("null read concern clears")
   {
      auto const rc = make_owning_ptr(mongoac_read_concern_new(), &mongoac_read_concern_destroy);
      mongoac_database_options_set_read_concern(opts, rc);
      mongoac_database_options_set_read_concern(opts, nullptr);
      SUCCEED();
   }

   SECTION("valid")
   {
      auto const rc = make_owning_ptr(mongoac_read_concern_new(), &mongoac_read_concern_destroy);
      mongoac_database_options_set_read_concern(opts, rc);
      SUCCEED();
   }
}

TEST_CASE("set_write_concern", "[mongoac][database_options]")
{
   auto const opts = make_owning_ptr(mongoac_database_options_new(), &mongoac_database_options_destroy);

   SECTION("null handle")
   {
      mongoac_database_options_set_write_concern(nullptr, nullptr);
      SUCCEED();
   }

   SECTION("null write concern clears")
   {
      auto const wc = make_owning_ptr(mongoac_write_concern_new(), &mongoac_write_concern_destroy);
      mongoac_database_options_set_write_concern(opts, wc);
      mongoac_database_options_set_write_concern(opts, nullptr);
      SUCCEED();
   }

   SECTION("valid")
   {
      auto const wc = make_owning_ptr(mongoac_write_concern_new(), &mongoac_write_concern_destroy);
      mongoac_database_options_set_write_concern(opts, wc);
      SUCCEED();
   }
}

TEST_CASE("set_read_preference", "[mongoac][database_options]")
{
   auto const opts = make_owning_ptr(mongoac_database_options_new(), &mongoac_database_options_destroy);

   SECTION("null handle")
   {
      mongoac_database_options_set_read_preference(nullptr, nullptr);
      SUCCEED();
   }

   SECTION("null read preference clears")
   {
      auto const rp = make_owning_ptr(mongoac_read_preference_new(), &mongoac_read_preference_destroy);
      mongoac_database_options_set_read_preference(opts, rp);
      mongoac_database_options_set_read_preference(opts, nullptr);
      SUCCEED();
   }

   SECTION("valid")
   {
      auto const rp = make_owning_ptr(mongoac_read_preference_new(), &mongoac_read_preference_destroy);
      mongoac_database_options_set_read_preference(opts, rp);
      SUCCEED();
   }
}
