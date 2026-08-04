#include <mongoac/read_preference.h>

#include <bson/bson.h>

#include <catch2/catch_test_macros.hpp>
#include <mongoac/error.h>
#include <test_util/bson.hh>
#include <test_util/error.hh>
#include <test_util/owning_ptr.hh>

using mongoac::test_util::bson_from_json;
using mongoac::test_util::make_owning_ptr;

TEST_CASE("new", "[mongoac][read_preference]")
{
   SECTION("default")
   {
      auto const rp = make_owning_ptr(mongoac_read_preference_new(), &mongoac_read_preference_destroy);
      REQUIRE(rp != nullptr);
   }
}

TEST_CASE("destroy", "[mongoac][read_preference]")
{
   SECTION("null")
   {
      mongoac_read_preference_destroy(nullptr);
      SUCCEED();
   }
}

TEST_CASE("set_primary", "[mongoac][read_preference]")
{
   auto const rp = make_owning_ptr(mongoac_read_preference_new(), &mongoac_read_preference_destroy);

   SECTION("null")
   {
      mongoac_read_preference_set_primary(nullptr);
      SUCCEED();
   }

   SECTION("valid")
   {
      mongoac_read_preference_set_primary(rp);
      SUCCEED();
   }
}

TEST_CASE("set_secondary", "[mongoac][read_preference]")
{
   auto const rp = make_owning_ptr(mongoac_read_preference_new(), &mongoac_read_preference_destroy);

   SECTION("null")
   {
      mongoac_read_preference_set_secondary(nullptr);
      SUCCEED();
   }

   SECTION("valid")
   {
      mongoac_read_preference_set_secondary(rp);
      SUCCEED();
   }
}

TEST_CASE("set_primary_preferred", "[mongoac][read_preference]")
{
   auto const rp = make_owning_ptr(mongoac_read_preference_new(), &mongoac_read_preference_destroy);

   SECTION("null")
   {
      mongoac_read_preference_set_primary_preferred(nullptr);
      SUCCEED();
   }

   SECTION("valid")
   {
      mongoac_read_preference_set_primary_preferred(rp);
      SUCCEED();
   }
}

TEST_CASE("set_secondary_preferred", "[mongoac][read_preference]")
{
   auto const rp = make_owning_ptr(mongoac_read_preference_new(), &mongoac_read_preference_destroy);

   SECTION("null")
   {
      mongoac_read_preference_set_secondary_preferred(nullptr);
      SUCCEED();
   }

   SECTION("valid")
   {
      mongoac_read_preference_set_secondary_preferred(rp);
      SUCCEED();
   }
}

TEST_CASE("set_nearest", "[mongoac][read_preference]")
{
   auto const rp = make_owning_ptr(mongoac_read_preference_new(), &mongoac_read_preference_destroy);

   SECTION("null")
   {
      mongoac_read_preference_set_nearest(nullptr);
      SUCCEED();
   }

   SECTION("valid")
   {
      mongoac_read_preference_set_nearest(rp);
      SUCCEED();
   }
}

TEST_CASE("set_max_staleness_secs", "[mongoac][read_preference]")
{
   auto const error = make_owning_ptr(mongoac_error_new(), &mongoac_error_destroy);
   auto const rp = make_owning_ptr(mongoac_read_preference_new(), &mongoac_read_preference_destroy);

   SECTION("null handle")
   {
      mongoac_read_preference_set_max_staleness_seconds(nullptr, 123, error);
      REQUIRE_MONGOAC_INVALID_ARGUMENT(error);
   }

   SECTION("rejects on primary")
   {
      // Default mode is Primary; options require a non-primary mode.
      mongoac_read_preference_set_max_staleness_seconds(rp, 123, error);
      REQUIRE_MONGOAC_INVALID_ARGUMENT(error);
   }

   SECTION("valid")
   {
      mongoac_read_preference_set_secondary(rp);
      mongoac_read_preference_set_max_staleness_seconds(rp, 123, error);
      CHECK_MONGOAC_OK(error);
   }
}

TEST_CASE("add_tag_set", "[mongoac][read_preference]")
{
   auto const error = make_owning_ptr(mongoac_error_new(), &mongoac_error_destroy);
   auto const rp = make_owning_ptr(mongoac_read_preference_new(), &mongoac_read_preference_destroy);

   SECTION("null handle")
   {
      auto const t = make_owning_ptr(bson_from_json(R"({"region": "us-east"})"), &bson_destroy);
      mongoac_read_preference_add_tag_set(nullptr, t, error);
      REQUIRE_MONGOAC_INVALID_ARGUMENT(error);
   }

   SECTION("rejects on primary")
   {
      auto const t = make_owning_ptr(bson_from_json(R"({"region": "us-east"})"), &bson_destroy);
      mongoac_read_preference_add_tag_set(rp, t, error);
      REQUIRE_MONGOAC_INVALID_ARGUMENT(error);
   }

   SECTION("null tag set document")
   {
      mongoac_read_preference_set_secondary(rp);
      mongoac_read_preference_add_tag_set(rp, nullptr, error);
      REQUIRE_MONGOAC_INVALID_ARGUMENT(error);
   }

   SECTION("valid tag sets")
   {
      mongoac_read_preference_set_secondary(rp);

      auto const t0 = make_owning_ptr(bson_from_json(R"({"region": "us-east"})"), &bson_destroy);
      mongoac_read_preference_add_tag_set(rp, t0, error);
      CHECK_MONGOAC_OK(error);

      auto const t1 = make_owning_ptr(bson_from_json(R"({"region": "us-west"})"), &bson_destroy);
      mongoac_read_preference_add_tag_set(rp, t1, error);
      CHECK_MONGOAC_OK(error);
   }

   SECTION("invalid tag set")
   {
      mongoac_read_preference_set_secondary(rp);

      auto const t = make_owning_ptr(bson_from_json(R"({"x": 1})"), &bson_destroy);
      mongoac_read_preference_add_tag_set(rp, t, error);
      CHECK_MONGOAC_ERROR_CATEGORY(error, MONGOAC_ERROR_CATEGORY_BSON);
   }
}

TEST_CASE("set_hedge_enabled", "[mongoac][read_preference]")
{
   auto const error = make_owning_ptr(mongoac_error_new(), &mongoac_error_destroy);
   auto const rp = make_owning_ptr(mongoac_read_preference_new(), &mongoac_read_preference_destroy);

   SECTION("null handle")
   {
      mongoac_read_preference_set_hedge_enabled(nullptr, true, error);
      REQUIRE_MONGOAC_INVALID_ARGUMENT(error);
   }

   SECTION("rejects on primary")
   {
      mongoac_read_preference_set_hedge_enabled(rp, true, error);
      REQUIRE_MONGOAC_INVALID_ARGUMENT(error);
   }

   SECTION("set true")
   {
      mongoac_read_preference_set_secondary(rp);
      mongoac_read_preference_set_hedge_enabled(rp, true, error);
      CHECK_MONGOAC_OK(error);
   }

   SECTION("set false")
   {
      mongoac_read_preference_set_secondary(rp);
      mongoac_read_preference_set_hedge_enabled(rp, false, error);
      CHECK_MONGOAC_OK(error);
   }
}
