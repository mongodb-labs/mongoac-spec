#include <mongoac/list_collections_options.h>

#include <bson/bson.h>

#include <catch2/catch_test_macros.hpp>
#include <mongoac/error.h>
#include <test_util/bson.hh>
#include <test_util/error.hh>
#include <test_util/owning_ptr.hh>

#include <cstdint>

using mongoac::test_util::bson_from_json;
using mongoac::test_util::make_owning_ptr;

TEST_CASE("new", "[mongoac][list_collections_options]")
{
   SECTION("default")
   {
      auto const opts =
         make_owning_ptr(mongoac_list_collections_options_new(), &mongoac_list_collections_options_destroy);
      REQUIRE(opts != nullptr);
   }
}

TEST_CASE("destroy", "[mongoac][list_collections_options]")
{
   SECTION("null")
   {
      mongoac_list_collections_options_destroy(nullptr);
      SUCCEED();
   }
}

TEST_CASE("set_batch_size", "[mongoac][list_collections_options]")
{
   auto const opts = make_owning_ptr(mongoac_list_collections_options_new(), &mongoac_list_collections_options_destroy);

   SECTION("null")
   {
      mongoac_list_collections_options_set_batch_size(nullptr, 100);
      SUCCEED();
   }

   SECTION("valid")
   {
      mongoac_list_collections_options_set_batch_size(opts, 100);
      SUCCEED();
   }
}

TEST_CASE("set_authorized_collections", "[mongoac][list_collections_options]")
{
   auto const opts = make_owning_ptr(mongoac_list_collections_options_new(), &mongoac_list_collections_options_destroy);

   SECTION("null")
   {
      mongoac_list_collections_options_set_authorized_collections(nullptr, true);
      SUCCEED();
   }

   SECTION("set true")
   {
      mongoac_list_collections_options_set_authorized_collections(opts, true);
      SUCCEED();
   }

   SECTION("set false")
   {
      mongoac_list_collections_options_set_authorized_collections(opts, false);
      SUCCEED();
   }
}

TEST_CASE("set_filter", "[mongoac][list_collections_options]")
{
   auto const error = make_owning_ptr(mongoac_error_new(), &mongoac_error_destroy);
   auto const opts = make_owning_ptr(mongoac_list_collections_options_new(), &mongoac_list_collections_options_destroy);

   SECTION("null handle")
   {
      auto const filter = make_owning_ptr(bson_from_json(R"({"x": 1})"), &bson_destroy);
      mongoac_list_collections_options_set_filter(nullptr, filter, error);
      REQUIRE_MONGOAC_INVALID_ARGUMENT(error);
   }

   SECTION("null bson clears")
   {
      mongoac_list_collections_options_set_filter(opts, nullptr, error);
      CHECK_MONGOAC_OK(error);
   }

   SECTION("invalid")
   {
      std::uint8_t data[] = {12, 0, 0, 0, 16, 'x', '\0', 1, 0, 0, 0, 0}; // {"x": 1}
      bson_t doc = {};
      REQUIRE(bson_init_static(&doc, data, sizeof(data)));
      data[sizeof(data) - 1] = 1; // Corruption.

      mongoac_list_collections_options_set_filter(opts, &doc, error);
      CHECK_FALSE_MONGOAC_OK(error);
      CHECK_MONGOAC_ERROR_CATEGORY(error, MONGOAC_ERROR_CATEGORY_BSON);
   }

   SECTION("valid")
   {
      auto const filter = make_owning_ptr(bson_from_json(R"({"x": 1})"), &bson_destroy);
      mongoac_list_collections_options_set_filter(opts, filter, error);
      CHECK_MONGOAC_OK(error);
   }
}

TEST_CASE("set_comment", "[mongoac][list_collections_options]")
{
   auto const error = make_owning_ptr(mongoac_error_new(), &mongoac_error_destroy);
   auto const opts = make_owning_ptr(mongoac_list_collections_options_new(), &mongoac_list_collections_options_destroy);

   SECTION("null handle")
   {
      auto const comment = make_owning_ptr(bson_from_json(R"({"x": 1})"), &bson_destroy);
      mongoac_list_collections_options_set_comment(nullptr, comment, error);
      REQUIRE_MONGOAC_INVALID_ARGUMENT(error);
   }

   SECTION("null bson clears")
   {
      mongoac_list_collections_options_set_comment(opts, nullptr, error);
      CHECK_MONGOAC_OK(error);
   }

   SECTION("invalid")
   {
      std::uint8_t data[] = {12, 0, 0, 0, 16, 'x', '\0', 1, 0, 0, 0, 0}; // {"x": 1}
      bson_t doc = {};
      REQUIRE(bson_init_static(&doc, data, sizeof(data)));
      data[sizeof(data) - 1] = 1; // Corruption.

      mongoac_list_collections_options_set_comment(opts, &doc, error);
      CHECK_FALSE_MONGOAC_OK(error);
      CHECK_MONGOAC_ERROR_CATEGORY(error, MONGOAC_ERROR_CATEGORY_BSON);
   }

   SECTION("valid")
   {
      auto const comment = make_owning_ptr(bson_from_json(R"({"x": 1})"), &bson_destroy);
      mongoac_list_collections_options_set_comment(opts, comment, error);
      CHECK_MONGOAC_OK(error);
   }
}
