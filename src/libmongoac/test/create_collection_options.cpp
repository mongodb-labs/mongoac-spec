#include <mongoac/create_collection_options.h>

#include <bson/bson.h>

#include <catch2/catch_test_macros.hpp>
#include <mongoac/error.h>
#include <test_util/bson.hh>
#include <test_util/error.hh>
#include <test_util/owning_ptr.hh>

#include <cstdint>

using mongoac::test_util::bson_from_json;
using mongoac::test_util::make_owning_ptr;

TEST_CASE("create_collection_options_new", "[mongoac][create_collection_options]")
{
   SECTION("default")
   {
      auto const opts =
         make_owning_ptr(mongoac_create_collection_options_new(), &mongoac_create_collection_options_destroy);
      REQUIRE(opts != nullptr);
   }
}

TEST_CASE("create_collection_options_destroy", "[mongoac][create_collection_options]")
{
   SECTION("null")
   {
      mongoac_create_collection_options_destroy(nullptr);
      SUCCEED();
   }
}

TEST_CASE("create_collection_options_set_from_bson", "[mongoac][create_collection_options]")
{
   auto const error = make_owning_ptr(mongoac_error_new(), &mongoac_error_destroy);

   auto const opts =
      make_owning_ptr(mongoac_create_collection_options_new(), &mongoac_create_collection_options_destroy);

   SECTION("null handle")
   {
      auto const bson = make_owning_ptr(bson_from_json(R"({"capped": true})"), &bson_destroy);
      mongoac_create_collection_options_set_from_bson(nullptr, make_bson_view(bson), error);
      REQUIRE_MONGOAC_INVALID_ARGUMENT(error);
   }

   SECTION("null bson")
   {
      mongoac_create_collection_options_set_from_bson(opts, {}, error);
      CHECK_MONGOAC_OK(error);
   }

   SECTION("invalid")
   {
      std::uint8_t data[] = {12, 0, 0, 0, 16, 'x', '\0', 1, 0, 0, 0, 1}; // {"x": 1} with last-byte corruption.

      mongoac_create_collection_options_set_from_bson(opts, {data, sizeof(data)}, error);
      CHECK_FALSE_MONGOAC_OK(error);
      CHECK_MONGOAC_ERROR_CATEGORY(error, MONGOAC_ERROR_CATEGORY_BSON);
   }

   SECTION("valid")
   {
      auto const bson = make_owning_ptr(bson_from_json(R"({"comment": {"x": 1}})"), &bson_destroy);
      mongoac_create_collection_options_set_from_bson(opts, make_bson_view(bson), error);
      CHECK_MONGOAC_OK(error);
   }
}
