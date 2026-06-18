#include <mongoac/client_options.h>

#include <bson/bson.h>

#include <catch2/catch_test_macros.hpp>
#include <catch2/matchers/catch_matchers_string.hpp>
#include <mongoac/client.h>
#include <mongoac/error.h>

#include <cstring>

TEST_CASE("new", "[mongoac][client_options]")
{
   SECTION("default")
   {
      auto const opts = mongoac_client_options_new();
      REQUIRE(opts != nullptr);
      mongoac_client_options_destroy(opts);
   }
}

TEST_CASE("destroy", "[mongoac][client_options]")
{
   SECTION("null")
   {
      mongoac_client_options_destroy(nullptr);
      SUCCEED();
   }
}

TEST_CASE("set_capture_command_events", "[mongoac][client_options]")
{
   auto const opts = mongoac_client_options_new();
   REQUIRE(opts != nullptr);

   SECTION("null")
   {
      mongoac_client_options_set_capture_command_events(nullptr, true);
      SUCCEED();
   }

   SECTION("set true")
   {
      mongoac_client_options_set_capture_command_events(opts, true);
      SUCCEED();
   }

   SECTION("set false")
   {
      mongoac_client_options_set_capture_command_events(opts, false);
      SUCCEED();
   }

   mongoac_client_options_destroy(opts);
}

static bson_t *
build_server_api(const char *version, bool set_strict, bool strict_val, bool set_deprecation, bool deprecation_val)
{
   return BCON_NEW("apiVersion",
                   BCON_UTF8(version),
                   "apiStrict",
                   BCON_BOOL(set_strict ? strict_val : false),
                   "apiDeprecationErrors",
                   BCON_BOOL(set_deprecation ? deprecation_val : false));
}

TEST_CASE("set_server_api", "[mongoac][client_options]")
{
   auto const opts = mongoac_client_options_new();
   REQUIRE(opts != nullptr);

   SECTION("null")
   {
      auto const error = mongoac_error_new();
      auto const bson = build_server_api("1", false, false, false, false);

      mongoac_client_options_set_server_api(nullptr, bson, error);
      CHECK(mongoac_error_category(error) == MONGOAC_ERROR_CATEGORY_MONGOAC);
      CHECK(mongoac_error_code(error) == MONGOAC_ERROR_CODE_INVALID_ARGUMENT);

      bson_destroy(bson);
      mongoac_error_destroy(error);
   }

   SECTION("null ")
   {
      auto const error = mongoac_error_new();

      mongoac_client_options_set_server_api(opts, nullptr, error);
      CHECK(mongoac_error_category(error) == MONGOAC_ERROR_CATEGORY_NONE);

      mongoac_error_destroy(error);
   }

   SECTION("valid")
   {
      auto const bson = build_server_api("1", false, false, false, false);

      mongoac_client_options_set_server_api(opts, bson, nullptr);

      bson_destroy(bson);
   }

   SECTION("strict")
   {
      auto const bson = build_server_api("1", true, true, false, false);

      mongoac_client_options_set_server_api(opts, bson, nullptr);

      bson_destroy(bson);
   }

   SECTION("all valid")
   {
      auto const bson = build_server_api("1", true, true, true, true);

      mongoac_client_options_set_server_api(opts, bson, nullptr);

      bson_destroy(bson);
   }

   SECTION("invalid apiVersion")
   {
      auto const error = mongoac_error_new();
      auto const bson = build_server_api("2", false, false, false, false);

      mongoac_client_options_set_server_api(opts, bson, error);

      bson_destroy(bson);
      mongoac_error_destroy(error);
   }

   SECTION("invalid")
   {
      auto const error = mongoac_error_new();
      bson_t bson = BSON_INITIALIZER;

      mongoac_client_options_set_server_api(opts, &bson, error);

      bson_destroy(&bson);
      mongoac_error_destroy(error);
   }

   SECTION("wrong type for apiVersion")
   {
      auto const error = mongoac_error_new();
      auto const bson = BCON_NEW("apiVersion", BCON_BOOL(true));

      mongoac_client_options_set_server_api(opts, bson, error);

      bson_destroy(bson);
      mongoac_error_destroy(error);
   }

   mongoac_client_options_destroy(opts);
}
