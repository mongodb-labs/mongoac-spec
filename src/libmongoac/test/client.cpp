#include <mongoac/client.h>

#include <catch2/catch_test_macros.hpp>
#include <catch2/matchers/catch_matchers_string.hpp>
#include <mongoac/client_options.h>
#include <mongoac/error.h>
#include <mongoac/future.h>
#include <mongoac/runtime.h>
#include <mongoac/server_api.h>
#include <test_util/bson.hh>
#include <test_util/owning_ptr.hh>
#include <test_util/string.hh>

using mongoac::test_util::make_owning_ptr;
using mongoac::test_util::owning_bson;
using mongoac::test_util::to_string;

TEST_CASE("new", "[mongoac][client]")
{
   SECTION("nullptr")
   {
      auto const error = make_owning_ptr(mongoac_error_new(), &mongoac_error_destroy);
      auto const client = mongoac_client_new(nullptr, error);

      CHECK(client == nullptr);

      CHECK(mongoac_error_category(error) == MONGOAC_ERROR_CATEGORY_MONGOAC);
      CHECK(to_string(mongoac_error_message(error)) != "");
   }

   SECTION("invalid URI")
   {
      auto const error = make_owning_ptr(mongoac_error_new(), &mongoac_error_destroy);
      auto const client = mongoac_client_new("not-a-uri", error);

      CHECK(client == nullptr);

      CHECK(mongoac_error_category(error) == MONGOAC_ERROR_CATEGORY_RUST);
      CHECK(to_string(mongoac_error_message(error)) != "");
   }

   SECTION("invalid UTF-8")
   {
      auto const error = make_owning_ptr(mongoac_error_new(), &mongoac_error_destroy);
      auto const client = mongoac_client_new("\x80", error);

      CHECK(client == nullptr);
      CHECK(mongoac_error_category(error) == MONGOAC_ERROR_CATEGORY_MONGOAC);
      CHECK(mongoac_error_code(error) == MONGOAC_ERROR_CODE_INVALID_ARGUMENT);
      CHECK_THAT(to_string(mongoac_error_message(error)), Catch::Matchers::ContainsSubstring("UTF-8"));
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
      auto const error = make_owning_ptr(mongoac_error_new(), &mongoac_error_destroy);
      auto const client = make_owning_ptr(mongoac_client_new_with_options(nullptr, error), &mongoac_client_destroy);
      CHECK(client != nullptr);
      CHECK(mongoac_error_code(error) == MONGOAC_ERROR_CODE_OK);
      CHECK(mongoac_client_count_command_events(client) == 0);
   }

   SECTION("default options")
   {
      auto const opts = make_owning_ptr(mongoac_client_options_new(), &mongoac_client_options_destroy);
      REQUIRE(opts != nullptr);

      auto const client = make_owning_ptr(mongoac_client_new_with_options(opts, nullptr), &mongoac_client_destroy);
      CHECK(client != nullptr);
      CHECK(mongoac_client_count_command_events(client) == 0);
   }

   SECTION("capture_command_events")
   {
      auto const opts = make_owning_ptr(mongoac_client_options_new(), &mongoac_client_options_destroy);
      REQUIRE(opts != nullptr);
      mongoac_client_options_set_capture_command_events(opts, true);

      auto const client = make_owning_ptr(mongoac_client_new_with_options(opts, nullptr), &mongoac_client_destroy);
      CHECK(client != nullptr);
      CHECK(mongoac_client_count_command_events(client) == 0);
   }

   SECTION("server_api")
   {
      auto const opts = make_owning_ptr(mongoac_client_options_new(), &mongoac_client_options_destroy);
      REQUIRE(opts != nullptr);

      auto const api = make_owning_ptr(mongoac_server_api_new(), &mongoac_server_api_destroy);
      mongoac_server_api_set_strict(api, true);
      mongoac_client_options_set_server_api(opts, api, nullptr);

      auto const client = make_owning_ptr(mongoac_client_new_with_options(opts, nullptr), &mongoac_client_destroy);
      CHECK(client != nullptr);
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
      auto const client =
         make_owning_ptr(mongoac_client_new("mongodb://localhost:27017", nullptr), &mongoac_client_destroy);
      REQUIRE(client != nullptr);

      auto const runtime = make_owning_ptr(mongoac_client_get_runtime(client), &mongoac_runtime_destroy);
      CHECK(runtime != nullptr);
   }
}

TEST_CASE("append_metadata", "[mongoac][client]")
{
   auto const client = mongoac_client_new("mongodb://localhost:27017", nullptr);
   REQUIRE(client != nullptr);

   SECTION("null")
   {
      auto const error = make_owning_ptr(mongoac_error_new(), &mongoac_error_destroy);
      mongoac_client_append_metadata(nullptr, "name", nullptr, nullptr, error);

      CHECK(mongoac_error_category(error) == MONGOAC_ERROR_CATEGORY_MONGOAC);
      CHECK(mongoac_error_code(error) == MONGOAC_ERROR_CODE_INVALID_ARGUMENT);
   }

   SECTION("null name")
   {
      auto const error = make_owning_ptr(mongoac_error_new(), &mongoac_error_destroy);
      mongoac_client_append_metadata(client, nullptr, nullptr, nullptr, error);

      CHECK(mongoac_error_category(error) == MONGOAC_ERROR_CATEGORY_NONE);
      CHECK(mongoac_error_code(error) == MONGOAC_ERROR_CODE_OK);
   }

   SECTION("with delimiter")
   {
      auto const error = make_owning_ptr(mongoac_error_new(), &mongoac_error_destroy);
      mongoac_client_append_metadata(client, "bad|name", nullptr, nullptr, error);

      CHECK(mongoac_error_category(error) == MONGOAC_ERROR_CATEGORY_NONE);
      CHECK(mongoac_error_code(error) == MONGOAC_ERROR_CODE_OK);
   }

   SECTION("invalid UTF-8 name")
   {
      auto const error = make_owning_ptr(mongoac_error_new(), &mongoac_error_destroy);
      mongoac_client_append_metadata(client, "\x80", nullptr, nullptr, error);

      CHECK(mongoac_error_category(error) == MONGOAC_ERROR_CATEGORY_MONGOAC);
      CHECK(mongoac_error_code(error) == MONGOAC_ERROR_CODE_INVALID_ARGUMENT);
   }

   SECTION("invalid UTF-8 version")
   {
      auto const error = make_owning_ptr(mongoac_error_new(), &mongoac_error_destroy);
      mongoac_client_append_metadata(client, "wrapper", "\x80", nullptr, error);

      CHECK(mongoac_error_category(error) == MONGOAC_ERROR_CATEGORY_MONGOAC);
      CHECK(mongoac_error_code(error) == MONGOAC_ERROR_CODE_INVALID_ARGUMENT);
   }

   SECTION("valid name")
   {
      auto const error = make_owning_ptr(mongoac_error_new(), &mongoac_error_destroy);
      mongoac_client_append_metadata(client, "wrapper", nullptr, nullptr, error);

      CHECK(mongoac_error_category(error) == MONGOAC_ERROR_CATEGORY_NONE);
      CHECK(mongoac_error_code(error) == MONGOAC_ERROR_CODE_OK);
   }

   SECTION("all valid")
   {
      auto const error = make_owning_ptr(mongoac_error_new(), &mongoac_error_destroy);
      mongoac_client_append_metadata(client, "wrapper", "1.2.3", "linux", error);

      CHECK(mongoac_error_category(error) == MONGOAC_ERROR_CATEGORY_NONE);
      CHECK(mongoac_error_code(error) == MONGOAC_ERROR_CODE_OK);
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
   auto const error = make_owning_ptr(mongoac_error_new(), &mongoac_error_destroy);

   SECTION("null")
   {
      mongoac_client_shutdown(nullptr, error);

      CHECK(mongoac_error_category(error) == MONGOAC_ERROR_CATEGORY_MONGOAC);
      CHECK(mongoac_error_code(error) == MONGOAC_ERROR_CODE_INVALID_ARGUMENT);
   }

   SECTION("valid client")
   {
      auto const client =
         make_owning_ptr(mongoac_client_new("mongodb://localhost:27017", nullptr), &mongoac_client_destroy);
      REQUIRE(client != nullptr);

      mongoac_client_shutdown(client, error);

      CHECK(mongoac_error_category(error) == MONGOAC_ERROR_CATEGORY_NONE);
      CHECK(mongoac_error_code(error) == MONGOAC_ERROR_CODE_OK);

      {
         CHECK_FALSE(owning_bson(mongoac_client_list_databases(client, nullptr, nullptr, error)));
         CHECK(mongoac_error_category(error) == MONGOAC_ERROR_CATEGORY_RUST);
         CHECK(mongoac_error_code(error) != MONGOAC_ERROR_CODE_OK);
         CHECK_THAT(to_string(mongoac_error_message(error)), Catch::Matchers::ContainsSubstring("shut down"));
      }
   }
}

TEST_CASE("shutdown_async", "[mongoac][client]")
{
   SECTION("null")
   {
      auto const error = make_owning_ptr(mongoac_error_new(), &mongoac_error_destroy);
      auto const future = make_owning_ptr(mongoac_client_shutdown_async(nullptr, error), &mongoac_future_destroy);

      CHECK(future == nullptr);
      CHECK(mongoac_error_category(error) == MONGOAC_ERROR_CATEGORY_MONGOAC);
      CHECK(mongoac_error_code(error) == MONGOAC_ERROR_CODE_INVALID_ARGUMENT);
   }

   SECTION("valid client")
   {
      auto const client =
         make_owning_ptr(mongoac_client_new("mongodb://localhost:27017", nullptr), &mongoac_client_destroy);
      REQUIRE(client != nullptr);

      auto const runtime = make_owning_ptr(mongoac_client_get_runtime(client), &mongoac_runtime_destroy);
      REQUIRE(runtime != nullptr);

      auto const error = make_owning_ptr(mongoac_error_new(), &mongoac_error_destroy);
      auto const future = make_owning_ptr(mongoac_client_shutdown_async(client, error), &mongoac_future_destroy);
      REQUIRE(future != nullptr);
      CHECK(mongoac_error_code(error) == MONGOAC_ERROR_CODE_OK);

      mongoac_runtime_block_on(runtime, future, error);
      CHECK(mongoac_error_code(error) == MONGOAC_ERROR_CODE_OK);

      mongoac_future_get_void(future, error);
      CHECK(mongoac_error_code(error) == MONGOAC_ERROR_CODE_OK);

      {
         CHECK_FALSE(owning_bson(mongoac_client_list_databases(client, nullptr, nullptr, error)));
         CHECK(mongoac_error_category(error) == MONGOAC_ERROR_CATEGORY_RUST);
         CHECK(mongoac_error_code(error) != MONGOAC_ERROR_CODE_OK);
         CHECK_THAT(to_string(mongoac_error_message(error)), Catch::Matchers::ContainsSubstring("shut down"));
      }
   }
}
