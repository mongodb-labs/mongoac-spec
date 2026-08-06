#include <mongoac/collection_options.h>

#include <catch2/catch_test_macros.hpp>
#include <mongoac/collection.h>
#include <mongoac/database.h>
#include <mongoac/error.h>
#include <mongoac/read_concern.h>
#include <mongoac/read_preference.h>
#include <mongoac/server_selector.h>
#include <mongoac/write_concern.h>
#include <test_util/error.hh>
#include <test_util/owning_ptr.hh>

using mongoac::test_util::make_owning_ptr;

static bool
coll_options_predicate(mongoac_server_info_t const *info, void *user_data)
{
   (void)info;
   (void)user_data;
   return true;
}

TEST_CASE("new", "[mongoac][collection_options]")
{
   SECTION("default")
   {
      auto const opts = make_owning_ptr(mongoac_collection_options_new(), &mongoac_collection_options_destroy);
      REQUIRE(opts != nullptr);
   }
}

TEST_CASE("destroy", "[mongoac][collection_options]")
{
   SECTION("null")
   {
      mongoac_collection_options_destroy(nullptr);
      SUCCEED();
   }
}

TEST_CASE("set_read_concern", "[mongoac][collection_options]")
{
   auto const opts = make_owning_ptr(mongoac_collection_options_new(), &mongoac_collection_options_destroy);

   SECTION("null handle")
   {
      mongoac_collection_options_set_read_concern(nullptr, nullptr);
      SUCCEED();
   }

   SECTION("null read concern clears")
   {
      auto const rc = make_owning_ptr(mongoac_read_concern_new(), &mongoac_read_concern_destroy);
      mongoac_collection_options_set_read_concern(opts, rc);
      mongoac_collection_options_set_read_concern(opts, nullptr);
      SUCCEED();
   }

   SECTION("valid")
   {
      auto const rc = make_owning_ptr(mongoac_read_concern_new(), &mongoac_read_concern_destroy);
      mongoac_collection_options_set_read_concern(opts, rc);
      SUCCEED();
   }
}

TEST_CASE("set_write_concern", "[mongoac][collection_options]")
{
   auto const opts = make_owning_ptr(mongoac_collection_options_new(), &mongoac_collection_options_destroy);

   SECTION("null handle")
   {
      mongoac_collection_options_set_write_concern(nullptr, nullptr);
      SUCCEED();
   }

   SECTION("null write concern clears")
   {
      auto const wc = make_owning_ptr(mongoac_write_concern_new(), &mongoac_write_concern_destroy);
      mongoac_collection_options_set_write_concern(opts, wc);
      mongoac_collection_options_set_write_concern(opts, nullptr);
      SUCCEED();
   }

   SECTION("valid")
   {
      auto const wc = make_owning_ptr(mongoac_write_concern_new(), &mongoac_write_concern_destroy);
      mongoac_collection_options_set_write_concern(opts, wc);
      SUCCEED();
   }
}

TEST_CASE("set_read_preference", "[mongoac][collection_options]")
{
   auto const opts = make_owning_ptr(mongoac_collection_options_new(), &mongoac_collection_options_destroy);

   SECTION("null handle")
   {
      mongoac_collection_options_set_read_preference(nullptr, nullptr);
      SUCCEED();
   }

   SECTION("null read preference clears")
   {
      auto const rp = make_owning_ptr(mongoac_read_preference_new(), &mongoac_read_preference_destroy);
      mongoac_collection_options_set_read_preference(opts, rp);
      mongoac_collection_options_set_read_preference(opts, nullptr);
      SUCCEED();
   }

   SECTION("valid")
   {
      auto const rp = make_owning_ptr(mongoac_read_preference_new(), &mongoac_read_preference_destroy);
      mongoac_collection_options_set_read_preference(opts, rp);
      SUCCEED();
   }
}

TEST_CASE("set_server_selector", "[mongoac][collection_options]")
{
   auto const opts = make_owning_ptr(mongoac_collection_options_new(), &mongoac_collection_options_destroy);

   SECTION("null handle")
   {
      mongoac_collection_options_set_server_selector(nullptr, nullptr);
      SUCCEED();
   }

   SECTION("null server selector clears")
   {
      auto const sc = make_owning_ptr(mongoac_server_selector_new(&coll_options_predicate, nullptr),
                                      &mongoac_server_selector_destroy);
      mongoac_collection_options_set_server_selector(opts, sc);
      mongoac_collection_options_set_server_selector(opts, nullptr);
      SUCCEED();
   }

   SECTION("valid")
   {
      auto const sc = make_owning_ptr(mongoac_server_selector_new(&coll_options_predicate, nullptr),
                                      &mongoac_server_selector_destroy);
      mongoac_collection_options_set_server_selector(opts, sc);
      SUCCEED();
   }
}

TEST_CASE("get_collection_with_options", "[mongoac][collection_options]")
{
   auto const error = make_owning_ptr(mongoac_error_new(), &mongoac_error_destroy);
   auto const opts = make_owning_ptr(mongoac_collection_options_new(), &mongoac_collection_options_destroy);

   SECTION("null database returns null")
   {
      auto *coll = mongoac_database_get_collection_with_options(nullptr, "coll", opts, error);
      REQUIRE(coll == nullptr);
      REQUIRE_MONGOAC_INVALID_ARGUMENT(error);
   }

   SECTION("null options behaves like plain get_collection")
   {
      // Construction with null options should succeed the same as
      // mongoac_database_get_collection. Requires a client + database; tested
      // in collection.cpp integration tests with a live server.
      SUCCEED();
   }
}
