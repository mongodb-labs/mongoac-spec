#include <mongoac/client.h>

#include <bson/bson.h>

#include <catch2/catch_test_macros.hpp>
#include <catch2/matchers/catch_matchers_string.hpp>
#include <mongoac/client_options.h>
#include <mongoac/error.h>
#include <mongoac/future.h>
#include <mongoac/runtime.h>

TEST_CASE("new", "[mongoac][client]")
{
   SECTION("nullptr")
   {
      auto const error = mongoac_error_new();
      auto const client = mongoac_client_new(nullptr, error);

      CHECK(client == nullptr);

      CHECK(mongoac_error_category(error) == MONGOAC_ERROR_CATEGORY_MONGOAC);
      CHECK(mongoac_error_message(error) != nullptr);

      mongoac_error_destroy(error);
   }

   SECTION("invalid URI")
   {
      auto const error = mongoac_error_new();
      auto const client = mongoac_client_new("not-a-uri", error);

      CHECK(client == nullptr);

      CHECK(mongoac_error_category(error) == MONGOAC_ERROR_CATEGORY_RUST);
      CHECK(mongoac_error_message(error) != nullptr);

      mongoac_error_destroy(error);
   }

   SECTION("invalid UTF-8")
   {
      auto const error = mongoac_error_new();
      auto const client = mongoac_client_new("\x80", error);

      CHECK(client == nullptr);
      CHECK(mongoac_error_category(error) == MONGOAC_ERROR_CATEGORY_MONGOAC);
      CHECK(mongoac_error_code(error) == MONGOAC_ERROR_CODE_INVALID_ARGUMENT);
      CHECK_THAT(mongoac_error_message(error), Catch::Matchers::ContainsSubstring("UTF-8"));

      mongoac_error_destroy(error);
   }

   SECTION("valid URI")
   {
      auto const client = mongoac_client_new("mongodb://localhost:27017", nullptr);

      CHECK(client != nullptr);

      mongoac_client_destroy(client);
   }
}
TEST_CASE("client_new_with_options", "[mongoac][client_options]")
{
   SECTION("null")
   {
      auto const opts = mongoac_client_options_new();
      REQUIRE(opts != nullptr);
      auto const error = mongoac_error_new();

      auto const client = mongoac_client_new_with_options(nullptr, opts, error);
      CHECK(client == nullptr);
      CHECK(mongoac_error_category(error) == MONGOAC_ERROR_CATEGORY_MONGOAC);

      mongoac_error_destroy(error);
      mongoac_client_options_destroy(opts);
   }

   SECTION("invalid URI")
   {
      auto const opts = mongoac_client_options_new();
      REQUIRE(opts != nullptr);
      auto const error = mongoac_error_new();

      auto const client = mongoac_client_new_with_options("not-a-uri", opts, error);
      CHECK(client == nullptr);
      CHECK(mongoac_error_category(error) == MONGOAC_ERROR_CATEGORY_RUST);

      mongoac_error_destroy(error);
      mongoac_client_options_destroy(opts);
   }

   SECTION("null options")
   {
      auto const error = mongoac_error_new();

      auto const client = mongoac_client_new_with_options("mongodb://localhost:27017", nullptr, error);
      CHECK(client != nullptr);
      CHECK(mongoac_error_code(error) == MONGOAC_ERROR_CODE_OK);
      CHECK(mongoac_client_count_command_events(client) == 0);

      mongoac_client_destroy(client);
      mongoac_error_destroy(error);
   }

   SECTION("valid with default options")
   {
      auto const opts = mongoac_client_options_new();
      REQUIRE(opts != nullptr);

      auto const client = mongoac_client_new_with_options("mongodb://localhost:27017", opts, nullptr);
      CHECK(client != nullptr);
      CHECK(mongoac_client_count_command_events(client) == 0);

      mongoac_client_destroy(client);
      mongoac_client_options_destroy(opts);
   }

   SECTION("valid with command event capture enabled")
   {
      auto const opts = mongoac_client_options_new();
      REQUIRE(opts != nullptr);
      mongoac_client_options_set_capture_command_events(opts, true);

      auto const client = mongoac_client_new_with_options("mongodb://localhost:27017", opts, nullptr);
      CHECK(client != nullptr);
      CHECK(mongoac_client_count_command_events(client) == 0);

      mongoac_client_destroy(client);
      mongoac_client_options_destroy(opts);
   }

   SECTION("valid with server_api")
   {
      auto const opts = mongoac_client_options_new();
      REQUIRE(opts != nullptr);

      auto const bson = BCON_NEW("apiVersion", BCON_UTF8("1"), "apiStrict", BCON_BOOL(true));
      mongoac_client_options_set_server_api(opts, bson, nullptr);
      bson_destroy(bson);

      auto const client = mongoac_client_new_with_options("mongodb://localhost:27017", opts, nullptr);
      CHECK(client != nullptr);

      mongoac_client_destroy(client);
      mongoac_client_options_destroy(opts);
   }
}

TEST_CASE("get_runtime", "[mongoac][client]")
{
   SECTION("null")
   {
      auto const runtime = mongoac_client_get_runtime(nullptr);
      CHECK(runtime == nullptr);
   }

   SECTION("valid")
   {
      auto const client = mongoac_client_new("mongodb://localhost:27017", nullptr);
      REQUIRE(client != nullptr);

      auto const runtime = mongoac_client_get_runtime(client);
      CHECK(runtime != nullptr);

      mongoac_runtime_destroy(runtime);
      mongoac_client_destroy(client);
   }
}

TEST_CASE("append_metadata", "[mongoac][client]")
{
   auto const client = mongoac_client_new("mongodb://localhost:27017", nullptr);
   REQUIRE(client != nullptr);

   SECTION("null")
   {
      auto const error = mongoac_error_new();
      mongoac_client_append_metadata(nullptr, "name", nullptr, nullptr, error);

      CHECK(mongoac_error_category(error) == MONGOAC_ERROR_CATEGORY_MONGOAC);
      CHECK(mongoac_error_code(error) == MONGOAC_ERROR_CODE_INVALID_ARGUMENT);

      mongoac_error_destroy(error);
   }

   SECTION("null name")
   {
      auto const error = mongoac_error_new();
      mongoac_client_append_metadata(client, nullptr, nullptr, nullptr, error);

      CHECK(mongoac_error_category(error) == MONGOAC_ERROR_CATEGORY_NONE);
      CHECK(mongoac_error_code(error) == MONGOAC_ERROR_CODE_OK);

      mongoac_error_destroy(error);
   }

   SECTION("with delimiter")
   {
      auto const error = mongoac_error_new();
      mongoac_client_append_metadata(client, "bad|name", nullptr, nullptr, error);

      CHECK(mongoac_error_category(error) == MONGOAC_ERROR_CATEGORY_NONE);
      CHECK(mongoac_error_code(error) == MONGOAC_ERROR_CODE_OK);

      mongoac_error_destroy(error);
   }

   SECTION("invalid UTF-8 name")
   {
      auto const error = mongoac_error_new();
      mongoac_client_append_metadata(client, "\x80", nullptr, nullptr, error);

      CHECK(mongoac_error_category(error) == MONGOAC_ERROR_CATEGORY_MONGOAC);
      CHECK(mongoac_error_code(error) == MONGOAC_ERROR_CODE_INVALID_ARGUMENT);

      mongoac_error_destroy(error);
   }

   SECTION("invalid UTF-8 version")
   {
      auto const error = mongoac_error_new();
      mongoac_client_append_metadata(client, "wrapper", "\x80", nullptr, error);

      CHECK(mongoac_error_category(error) == MONGOAC_ERROR_CATEGORY_MONGOAC);
      CHECK(mongoac_error_code(error) == MONGOAC_ERROR_CODE_INVALID_ARGUMENT);

      mongoac_error_destroy(error);
   }

   SECTION("valid name")
   {
      auto const error = mongoac_error_new();
      mongoac_client_append_metadata(client, "wrapper", nullptr, nullptr, error);

      CHECK(mongoac_error_category(error) == MONGOAC_ERROR_CATEGORY_NONE);
      CHECK(mongoac_error_code(error) == MONGOAC_ERROR_CODE_OK);

      mongoac_error_destroy(error);
   }

   SECTION("all valid")
   {
      auto const error = mongoac_error_new();
      mongoac_client_append_metadata(client, "wrapper", "1.2.3", "linux", error);

      CHECK(mongoac_error_category(error) == MONGOAC_ERROR_CATEGORY_NONE);
      CHECK(mongoac_error_code(error) == MONGOAC_ERROR_CODE_OK);

      mongoac_error_destroy(error);
   }

   mongoac_client_destroy(client);
}

TEST_CASE("destroy", "[mongoac][client]")
{
   SECTION("null")
   {
      mongoac_client_destroy(nullptr);
      SUCCEED();
   }
}

TEST_CASE("shutdown", "[mongoac][client]")
{
   SECTION("null")
   {
      auto const error = mongoac_error_new();
      mongoac_client_shutdown(nullptr, error);

      CHECK(mongoac_error_category(error) == MONGOAC_ERROR_CATEGORY_MONGOAC);
      CHECK(mongoac_error_code(error) == MONGOAC_ERROR_CODE_INVALID_ARGUMENT);

      mongoac_error_destroy(error);
   }

   SECTION("valid client")
   {
      auto const client = mongoac_client_new("mongodb://localhost:27017", nullptr);
      REQUIRE(client != nullptr);

      auto const error = mongoac_error_new();
      mongoac_client_shutdown(client, error);

      CHECK(mongoac_error_category(error) == MONGOAC_ERROR_CATEGORY_NONE);
      CHECK(mongoac_error_code(error) == MONGOAC_ERROR_CODE_OK);

      mongoac_client_destroy(client);
      mongoac_error_destroy(error);
   }
}

TEST_CASE("shutdown_async", "[mongoac][client]")
{
   SECTION("null")
   {
      auto const error = mongoac_error_new();
      auto const future = mongoac_client_shutdown_async(nullptr, error);

      CHECK(future == nullptr);
      CHECK(mongoac_error_category(error) == MONGOAC_ERROR_CATEGORY_MONGOAC);
      CHECK(mongoac_error_code(error) == MONGOAC_ERROR_CODE_INVALID_ARGUMENT);

      mongoac_error_destroy(error);
   }

   SECTION("valid client")
   {
      auto const client = mongoac_client_new("mongodb://localhost:27017", nullptr);
      REQUIRE(client != nullptr);

      auto const runtime = mongoac_client_get_runtime(client);
      REQUIRE(runtime != nullptr);

      auto const error = mongoac_error_new();
      auto const future = mongoac_client_shutdown_async(client, error);
      REQUIRE(future != nullptr);
      CHECK(mongoac_error_code(error) == MONGOAC_ERROR_CODE_OK);

      mongoac_runtime_block_on(runtime, future, error);
      CHECK(mongoac_error_code(error) == MONGOAC_ERROR_CODE_OK);

      mongoac_future_get_void(future, error);
      CHECK(mongoac_error_code(error) == MONGOAC_ERROR_CODE_OK);

      mongoac_future_destroy(future);
      mongoac_runtime_destroy(runtime);
      mongoac_client_destroy(client);
      mongoac_error_destroy(error);
   }
}
