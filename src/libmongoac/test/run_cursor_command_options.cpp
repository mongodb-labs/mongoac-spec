#include <mongoac/run_cursor_command_options.h>

#include <bson/bson.h>

#include <catch2/catch_test_macros.hpp>
#include <mongoac/cursor_type.h>
#include <mongoac/error.h>
#include <mongoac/read_preference.h>
#include <test_util/bson.hh>
#include <test_util/error.hh>
#include <test_util/owning_ptr.hh>

#include <cstdint>

using mongoac::test_util::bson_from_json;
using mongoac::test_util::make_bson_view;
using mongoac::test_util::make_owning_ptr;

TEST_CASE("new", "[mongoac][run_cursor_command_options]")
{
   auto const opts =
      make_owning_ptr(mongoac_run_cursor_command_options_new(), &mongoac_run_cursor_command_options_destroy);
   REQUIRE(opts != nullptr);
}

TEST_CASE("destroy", "[mongoac][run_cursor_command_options]")
{
   SECTION("null")
   {
      mongoac_run_cursor_command_options_destroy(nullptr);
      SUCCEED();
   }
}

TEST_CASE("set_read_preference", "[mongoac][run_cursor_command_options]")
{
   auto const opts =
      make_owning_ptr(mongoac_run_cursor_command_options_new(), &mongoac_run_cursor_command_options_destroy);
   auto const rp = make_owning_ptr(mongoac_read_preference_new(), &mongoac_read_preference_destroy);

   SECTION("null handle")
   {
      mongoac_run_cursor_command_options_set_read_preference(nullptr, rp);
      SUCCEED();
   }

   SECTION("null preference clears")
   {
      mongoac_run_cursor_command_options_set_read_preference(opts, nullptr);
      SUCCEED();
   }

   SECTION("valid")
   {
      mongoac_run_cursor_command_options_set_read_preference(opts, rp);
      SUCCEED();
   }
}

TEST_CASE("set_server_selector", "[mongoac][run_cursor_command_options]")
{
   auto const opts =
      make_owning_ptr(mongoac_run_cursor_command_options_new(), &mongoac_run_cursor_command_options_destroy);

   SECTION("null handle")
   {
      mongoac_run_cursor_command_options_set_server_selector(nullptr, nullptr);
      SUCCEED();
   }

   SECTION("null selector clears")
   {
      mongoac_run_cursor_command_options_set_server_selector(opts, nullptr);
      SUCCEED();
   }
}

TEST_CASE("set_cursor_type", "[mongoac][run_cursor_command_options]")
{
   auto const opts =
      make_owning_ptr(mongoac_run_cursor_command_options_new(), &mongoac_run_cursor_command_options_destroy);

   SECTION("null handle")
   {
      mongoac_run_cursor_command_options_set_cursor_type(nullptr, MONGOAC_CURSOR_TYPE_NON_TAILABLE);
      SUCCEED();
   }

   SECTION("non-tailable")
   {
      mongoac_run_cursor_command_options_set_cursor_type(opts, MONGOAC_CURSOR_TYPE_NON_TAILABLE);
      SUCCEED();
   }

   SECTION("tailable")
   {
      mongoac_run_cursor_command_options_set_cursor_type(opts, MONGOAC_CURSOR_TYPE_TAILABLE);
      SUCCEED();
   }

   SECTION("tailable-await")
   {
      mongoac_run_cursor_command_options_set_cursor_type(opts, MONGOAC_CURSOR_TYPE_TAILABLE_AWAIT);
      SUCCEED();
   }
}

TEST_CASE("set_batch_size", "[mongoac][run_cursor_command_options]")
{
   auto const opts =
      make_owning_ptr(mongoac_run_cursor_command_options_new(), &mongoac_run_cursor_command_options_destroy);

   SECTION("null handle")
   {
      mongoac_run_cursor_command_options_set_batch_size(nullptr, 123);
      SUCCEED();
   }

   SECTION("valid")
   {
      mongoac_run_cursor_command_options_set_batch_size(opts, 123);
      SUCCEED();
   }
}

TEST_CASE("set_max_time_ms", "[mongoac][run_cursor_command_options]")
{
   auto const opts =
      make_owning_ptr(mongoac_run_cursor_command_options_new(), &mongoac_run_cursor_command_options_destroy);

   SECTION("null handle")
   {
      mongoac_run_cursor_command_options_set_max_time_ms(nullptr, 123);
      SUCCEED();
   }

   SECTION("valid")
   {
      mongoac_run_cursor_command_options_set_max_time_ms(opts, 123);
      SUCCEED();
   }
}

TEST_CASE("set_comment", "[mongoac][run_cursor_command_options]")
{
   auto const error = make_owning_ptr(mongoac_error_new(), &mongoac_error_destroy);
   auto const opts =
      make_owning_ptr(mongoac_run_cursor_command_options_new(), &mongoac_run_cursor_command_options_destroy);

   SECTION("null handle")
   {
      auto const comment = make_owning_ptr(bson_from_json(R"({"x": 1})"), &bson_destroy);
      mongoac_run_cursor_command_options_set_comment(nullptr, make_bson_view(comment), error);
      REQUIRE_MONGOAC_INVALID_ARGUMENT(error);
   }

   SECTION("null bson clears")
   {
      mongoac_run_cursor_command_options_set_comment(opts, {}, error);
      CHECK_MONGOAC_OK(error);
   }

   SECTION("invalid")
   {
      std::uint8_t data[] = {12, 0, 0, 0, 16, 'x', '\0', 1, 0, 0, 0, 1}; // {"x": 1} with last-byte corruption.

      mongoac_run_cursor_command_options_set_comment(opts, {data, sizeof(data)}, error);
      CHECK_FALSE_MONGOAC_OK(error);
      CHECK_MONGOAC_ERROR_CATEGORY(error, MONGOAC_ERROR_CATEGORY_BSON);
   }

   SECTION("valid")
   {
      auto const comment = make_owning_ptr(bson_from_json(R"({"x": 1})"), &bson_destroy);
      mongoac_run_cursor_command_options_set_comment(opts, make_bson_view(comment), error);
      CHECK_MONGOAC_OK(error);
   }
}
