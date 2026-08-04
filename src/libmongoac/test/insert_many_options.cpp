#include <mongoac/insert_many_options.h>

#include <bson/bson.h>

#include <catch2/catch_test_macros.hpp>
#include <mongoac/error.h>
#include <mongoac/write_concern.h>
#include <test_util/bson.hh>
#include <test_util/error.hh>
#include <test_util/owning_ptr.hh>

#include <cstdint>

using mongoac::test_util::bson_from_json;
using mongoac::test_util::make_owning_ptr;

TEST_CASE("new", "[mongoac][insert_many_options]")
{
   SECTION("default")
   {
      auto const opts = make_owning_ptr(mongoac_insert_many_options_new(), &mongoac_insert_many_options_destroy);
      REQUIRE(opts != nullptr);
   }
}

TEST_CASE("destroy", "[mongoac][insert_many_options]")
{
   SECTION("null")
   {
      mongoac_insert_many_options_destroy(nullptr);
      SUCCEED();
   }
}

TEST_CASE("set_bypass_document_validation", "[mongoac][insert_many_options]")
{
   auto const opts = make_owning_ptr(mongoac_insert_many_options_new(), &mongoac_insert_many_options_destroy);

   SECTION("null")
   {
      mongoac_insert_many_options_set_bypass_document_validation(nullptr, true);
      SUCCEED();
   }

   SECTION("set true")
   {
      mongoac_insert_many_options_set_bypass_document_validation(opts, true);
      SUCCEED();
   }

   SECTION("set false")
   {
      mongoac_insert_many_options_set_bypass_document_validation(opts, false);
      SUCCEED();
   }
}

TEST_CASE("set_ordered", "[mongoac][insert_many_options]")
{
   auto const opts = make_owning_ptr(mongoac_insert_many_options_new(), &mongoac_insert_many_options_destroy);

   SECTION("null")
   {
      mongoac_insert_many_options_set_ordered(nullptr, true);
      SUCCEED();
   }

   SECTION("set true")
   {
      mongoac_insert_many_options_set_ordered(opts, true);
      SUCCEED();
   }

   SECTION("set false")
   {
      mongoac_insert_many_options_set_ordered(opts, false);
      SUCCEED();
   }
}

TEST_CASE("set_write_concern", "[mongoac][insert_many_options]")
{
   auto const opts = make_owning_ptr(mongoac_insert_many_options_new(), &mongoac_insert_many_options_destroy);

   SECTION("null handle")
   {
      mongoac_insert_many_options_set_write_concern(nullptr, nullptr);
      SUCCEED();
   }

   SECTION("null write concern clears")
   {
      auto const wc = make_owning_ptr(mongoac_write_concern_new(), &mongoac_write_concern_destroy);
      mongoac_insert_many_options_set_write_concern(opts, wc);
      mongoac_insert_many_options_set_write_concern(opts, nullptr);
      SUCCEED();
   }

   SECTION("valid")
   {
      auto const wc = make_owning_ptr(mongoac_write_concern_new(), &mongoac_write_concern_destroy);
      mongoac_insert_many_options_set_write_concern(opts, wc);
      SUCCEED();
   }
}

TEST_CASE("set_comment", "[mongoac][insert_many_options]")
{
   auto const error = make_owning_ptr(mongoac_error_new(), &mongoac_error_destroy);
   auto const opts = make_owning_ptr(mongoac_insert_many_options_new(), &mongoac_insert_many_options_destroy);

   SECTION("null handle")
   {
      auto const comment = make_owning_ptr(bson_from_json(R"({"x": 1})"), &bson_destroy);
      mongoac_insert_many_options_set_comment(nullptr, comment, error);
      REQUIRE_MONGOAC_INVALID_ARGUMENT(error);
   }

   SECTION("null bson clears")
   {
      mongoac_insert_many_options_set_comment(opts, nullptr, error);
      CHECK_MONGOAC_OK(error);
   }

   SECTION("invalid")
   {
      std::uint8_t data[] = {12, 0, 0, 0, 16, 'x', '\0', 1, 0, 0, 0, 0}; // {"x": 1}
      bson_t doc = {};
      REQUIRE(bson_init_static(&doc, data, sizeof(data)));
      data[sizeof(data) - 1] = 1; // Corruption.

      mongoac_insert_many_options_set_comment(opts, &doc, error);
      CHECK_FALSE_MONGOAC_OK(error);
      CHECK_MONGOAC_ERROR_CATEGORY(error, MONGOAC_ERROR_CATEGORY_BSON);
   }

   SECTION("valid")
   {
      auto const comment = make_owning_ptr(bson_from_json(R"({"x": 1})"), &bson_destroy);
      mongoac_insert_many_options_set_comment(opts, comment, error);
      CHECK_MONGOAC_OK(error);
   }
}
