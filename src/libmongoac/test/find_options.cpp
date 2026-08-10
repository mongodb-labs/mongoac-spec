#include <mongoac/find_options.h>

#include <bson/bson.h>

#include <catch2/catch_test_macros.hpp>
#include <mongoac/error.h>
#include <test_util/bson.hh>
#include <test_util/error.hh>
#include <test_util/owning_ptr.hh>

#include <cstdint>

using mongoac::test_util::bson_from_json;
using mongoac::test_util::make_owning_ptr;

TEST_CASE("find_options_destroy", "[mongoac][find_options]")
{
   SECTION("null")
   {
      mongoac_find_options_destroy(nullptr);
      SUCCEED();
   }
}

TEST_CASE("find_options_new_from_bson", "[mongoac][find_options]")
{
   auto const error = make_owning_ptr(mongoac_error_new(), &mongoac_error_destroy);

   SECTION("null bson")
   {
      auto const opts = make_owning_ptr(mongoac_find_options_new_from_bson({}, error), &mongoac_find_options_destroy);
      CHECK_MONGOAC_OK(error);
      REQUIRE(opts != nullptr);
   }

   SECTION("invalid")
   {
      std::uint8_t data[] = {12, 0, 0, 0, 16, 'x', '\0', 1, 0, 0, 0, 1}; // {"x": 1} with last-byte corruption.

      auto const opts = mongoac_find_options_new_from_bson({data, sizeof(data)}, error);
      CHECK_FALSE_MONGOAC_OK(error);
      CHECK_MONGOAC_ERROR_CATEGORY(error, MONGOAC_ERROR_CATEGORY_BSON);
      REQUIRE(opts == nullptr);
   }

   SECTION("valid")
   {
      auto const bson = make_owning_ptr(bson_from_json(R"({"comment": {"x": 1}})"), &bson_destroy);
      auto const opts = make_owning_ptr(mongoac_find_options_new_from_bson(make_bson_view(bson), error),
                                        &mongoac_find_options_destroy);
      CHECK_MONGOAC_OK(error);
      REQUIRE(opts != nullptr);
   }
}
