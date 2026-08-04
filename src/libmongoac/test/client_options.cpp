#include <mongoac/client_options.h>

#include <catch2/catch_test_macros.hpp>
#include <mongoac/credential.h>
#include <mongoac/error.h>
#include <mongoac/read_concern.h>
#include <mongoac/read_preference.h>
#include <mongoac/server_api.h>
#include <mongoac/tls.h>
#include <mongoac/write_concern.h>
#include <test_util/error.hh>
#include <test_util/owning_ptr.hh>

using mongoac::test_util::make_owning_ptr;

TEST_CASE("new", "[mongoac][client_options]")
{
   SECTION("default")
   {
      auto const opts = make_owning_ptr(mongoac_client_options_new(), &mongoac_client_options_destroy);
      REQUIRE(opts != nullptr);
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
   auto const opts = make_owning_ptr(mongoac_client_options_new(), &mongoac_client_options_destroy);

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
}

TEST_CASE("set_server_api", "[mongoac][client_options]")
{
   auto const error = make_owning_ptr(mongoac_error_new(), &mongoac_error_destroy);
   auto const opts = make_owning_ptr(mongoac_client_options_new(), &mongoac_client_options_destroy);

   SECTION("null options")
   {
      auto const api = make_owning_ptr(mongoac_server_api_new(), &mongoac_server_api_destroy);
      mongoac_client_options_set_server_api(nullptr, api, error);
      CHECK_MONGOAC_INVALID_ARGUMENT(error);
   }

   SECTION("null api clears")
   {
      mongoac_client_options_set_server_api(opts, nullptr, error);
      CHECK_MONGOAC_OK(error);
   }

   SECTION("valid default")
   {
      auto const api = make_owning_ptr(mongoac_server_api_new(), &mongoac_server_api_destroy);

      mongoac_client_options_set_server_api(opts, api, error);
      CHECK_MONGOAC_OK(error);
   }

   SECTION("valid with strict")
   {
      auto const api = make_owning_ptr(mongoac_server_api_new(), &mongoac_server_api_destroy);
      mongoac_server_api_set_strict(api, true);

      mongoac_client_options_set_server_api(opts, api, error);
      CHECK_MONGOAC_OK(error);
   }

   SECTION("valid with strict and deprecation errors")
   {
      auto const api = make_owning_ptr(mongoac_server_api_new(), &mongoac_server_api_destroy);
      mongoac_server_api_set_strict(api, true);
      mongoac_server_api_set_deprecation_errors(api, true);

      mongoac_client_options_set_server_api(opts, api, error);
      CHECK_MONGOAC_OK(error);
   }
}

TEST_CASE("set_retry_reads", "[mongoac][client_options]")
{
   auto const opts = make_owning_ptr(mongoac_client_options_new(), &mongoac_client_options_destroy);

   SECTION("null")
   {
      mongoac_client_options_set_retry_reads(nullptr, true);
      SUCCEED();
   }

   SECTION("set true")
   {
      mongoac_client_options_set_retry_reads(opts, true);
      SUCCEED();
   }

   SECTION("set false")
   {
      mongoac_client_options_set_retry_reads(opts, false);
      SUCCEED();
   }
}

TEST_CASE("set_max_pool_size", "[mongoac][client_options]")
{
   auto const opts = make_owning_ptr(mongoac_client_options_new(), &mongoac_client_options_destroy);

   SECTION("null")
   {
      mongoac_client_options_set_max_pool_size(nullptr, 100);
      SUCCEED();
   }

   SECTION("valid")
   {
      mongoac_client_options_set_max_pool_size(opts, 100);
      SUCCEED();
   }
}

TEST_CASE("set_app_name", "[mongoac][client_options]")
{
   auto const error = make_owning_ptr(mongoac_error_new(), &mongoac_error_destroy);
   auto const opts = make_owning_ptr(mongoac_client_options_new(), &mongoac_client_options_destroy);

   SECTION("null options")
   {
      mongoac_client_options_set_app_name(nullptr, "app", error);
      CHECK_MONGOAC_INVALID_ARGUMENT(error);
   }

   SECTION("null string clears")
   {
      mongoac_client_options_set_app_name(opts, nullptr, error);
      CHECK_MONGOAC_OK(error);
   }

   SECTION("valid")
   {
      mongoac_client_options_set_app_name(opts, "app", error);
      CHECK_MONGOAC_OK(error);
   }
}

TEST_CASE("set_connect_timeout_ms", "[mongoac][client_options]")
{
   auto const opts = make_owning_ptr(mongoac_client_options_new(), &mongoac_client_options_destroy);

   SECTION("null")
   {
      mongoac_client_options_set_connect_timeout_ms(nullptr, 123);
      SUCCEED();
   }

   SECTION("valid")
   {
      mongoac_client_options_set_connect_timeout_ms(opts, 123);
      SUCCEED();
   }
}

TEST_CASE("set_server_monitoring_mode", "[mongoac][client_options]")
{
   auto const error = make_owning_ptr(mongoac_error_new(), &mongoac_error_destroy);
   auto const opts = make_owning_ptr(mongoac_client_options_new(), &mongoac_client_options_destroy);

   SECTION("null options")
   {
      mongoac_client_options_set_server_monitoring_mode(nullptr, MONGOAC_SERVER_MONITORING_MODE_STREAM, error);
      CHECK_MONGOAC_INVALID_ARGUMENT(error);
   }

   SECTION("valid stream")
   {
      mongoac_client_options_set_server_monitoring_mode(opts, MONGOAC_SERVER_MONITORING_MODE_STREAM, error);
      CHECK_MONGOAC_OK(error);
   }

   SECTION("valid poll")
   {
      mongoac_client_options_set_server_monitoring_mode(opts, MONGOAC_SERVER_MONITORING_MODE_POLL, error);
      CHECK_MONGOAC_OK(error);
   }

   SECTION("valid auto")
   {
      mongoac_client_options_set_server_monitoring_mode(opts, MONGOAC_SERVER_MONITORING_MODE_AUTO, error);
      CHECK_MONGOAC_OK(error);
   }

   SECTION("invalid mode")
   {
      mongoac_client_options_set_server_monitoring_mode(opts, -1, error);
      CHECK_MONGOAC_INVALID_ARGUMENT(error);
   }
}

TEST_CASE("set_read_concern", "[mongoac][client_options]")
{
   auto const opts = make_owning_ptr(mongoac_client_options_new(), &mongoac_client_options_destroy);

   SECTION("null handle clears")
   {
      mongoac_client_options_set_read_concern(opts, nullptr);
      SUCCEED();
   }

   SECTION("valid")
   {
      auto const rc = make_owning_ptr(mongoac_read_concern_new(), &mongoac_read_concern_destroy);
      mongoac_client_options_set_read_concern(opts, rc);
      SUCCEED();
   }
}

TEST_CASE("set_write_concern", "[mongoac][client_options]")
{
   auto const opts = make_owning_ptr(mongoac_client_options_new(), &mongoac_client_options_destroy);

   SECTION("null handle clears")
   {
      mongoac_client_options_set_write_concern(opts, nullptr);
      SUCCEED();
   }

   SECTION("valid")
   {
      auto const wc = make_owning_ptr(mongoac_write_concern_new(), &mongoac_write_concern_destroy);
      mongoac_client_options_set_write_concern(opts, wc);
      SUCCEED();
   }
}

TEST_CASE("set_read_preference", "[mongoac][client_options]")
{
   auto const opts = make_owning_ptr(mongoac_client_options_new(), &mongoac_client_options_destroy);

   SECTION("null handle clears")
   {
      mongoac_client_options_set_read_preference(opts, nullptr);
      SUCCEED();
   }

   SECTION("valid")
   {
      auto const rp = make_owning_ptr(mongoac_read_preference_new(), &mongoac_read_preference_destroy);
      mongoac_client_options_set_read_preference(opts, rp);
      SUCCEED();
   }
}

TEST_CASE("add_host", "[mongoac][client_options]")
{
   auto const opts = make_owning_ptr(mongoac_client_options_new(), &mongoac_client_options_destroy);

   SECTION("null options")
   {
      mongoac_client_options_add_host(nullptr, "localhost");
      SUCCEED();
   }

   SECTION("null host")
   {
      mongoac_client_options_add_host(opts, nullptr);
      SUCCEED();
   }

   SECTION("valid with port")
   {
      mongoac_client_options_add_host(opts, "localhost");
      SUCCEED();
   }
}

TEST_CASE("add_host_and_port", "[mongoac][client_options]")
{
   auto const opts = make_owning_ptr(mongoac_client_options_new(), &mongoac_client_options_destroy);

   SECTION("null options")
   {
      mongoac_client_options_add_host_and_port(nullptr, "localhost", 27017);
      SUCCEED();
   }

   SECTION("null host")
   {
      mongoac_client_options_add_host_and_port(opts, nullptr, 27017);
      SUCCEED();
   }

   SECTION("valid with port")
   {
      mongoac_client_options_add_host_and_port(opts, "localhost", 27018);
      SUCCEED();
   }

   SECTION("valid with port 0")
   {
      mongoac_client_options_add_host_and_port(opts, "localhost", 0);
      SUCCEED();
   }
}

TEST_CASE("add_compressor_snappy", "[mongoac][client_options]")
{
   auto const opts = make_owning_ptr(mongoac_client_options_new(), &mongoac_client_options_destroy);

   SECTION("null")
   {
      mongoac_client_options_add_compressor_snappy(nullptr);
      SUCCEED();
   }

   SECTION("valid")
   {
      mongoac_client_options_add_compressor_snappy(opts);
      SUCCEED();
   }
}

TEST_CASE("add_compressor_zlib", "[mongoac][client_options]")
{
   auto const opts = make_owning_ptr(mongoac_client_options_new(), &mongoac_client_options_destroy);

   SECTION("valid")
   {
      mongoac_client_options_add_compressor_zlib(opts);
      SUCCEED();
   }
}

TEST_CASE("add_compressor_zstd", "[mongoac][client_options]")
{
   auto const opts = make_owning_ptr(mongoac_client_options_new(), &mongoac_client_options_destroy);

   SECTION("valid")
   {
      mongoac_client_options_add_compressor_zstd(opts);
      SUCCEED();
   }
}

TEST_CASE("set_driver_info", "[mongoac][client_options]")
{
   auto const error = make_owning_ptr(mongoac_error_new(), &mongoac_error_destroy);
   auto const opts = make_owning_ptr(mongoac_client_options_new(), &mongoac_client_options_destroy);

   SECTION("null options")
   {
      mongoac_client_options_set_driver_info(nullptr, "lib", "1.0", "linux", error);
      CHECK_MONGOAC_INVALID_ARGUMENT(error);
   }

   SECTION("null name")
   {
      mongoac_client_options_set_driver_info(opts, nullptr, nullptr, nullptr, error);
      CHECK_MONGOAC_INVALID_ARGUMENT(error);
   }

   SECTION("valid name only")
   {
      mongoac_client_options_set_driver_info(opts, "driver", nullptr, nullptr, error);
      CHECK_MONGOAC_OK(error);
   }

   SECTION("valid with version and platform")
   {
      mongoac_client_options_set_driver_info(opts, "driver", "1.2.3", "platform", error);
      CHECK_MONGOAC_OK(error);
   }
}

TEST_CASE("set_tls", "[mongoac][client_options]")
{
   auto const opts = make_owning_ptr(mongoac_client_options_new(), &mongoac_client_options_destroy);

   SECTION("null handle clears")
   {
      mongoac_client_options_set_tls(opts, nullptr);
      SUCCEED();
   }

   SECTION("valid")
   {
      auto const tls = make_owning_ptr(mongoac_tls_options_new(), &mongoac_tls_options_destroy);
      mongoac_tls_options_set_allow_invalid_certificates(tls, true);
      mongoac_client_options_set_tls(opts, tls);
      SUCCEED();
   }
}

TEST_CASE("set_credential", "[mongoac][client_options]")
{
   auto const opts = make_owning_ptr(mongoac_client_options_new(), &mongoac_client_options_destroy);

   SECTION("null handle clears")
   {
      mongoac_client_options_set_credential(opts, nullptr);
      SUCCEED();
   }

   SECTION("valid")
   {
      auto const cred = make_owning_ptr(mongoac_credential_new(), &mongoac_credential_destroy);
      mongoac_credential_set_username(cred, "user", nullptr);
      mongoac_credential_set_mechanism(cred, MONGOAC_AUTH_MECHANISM_SCRAM_SHA_256, nullptr);
      mongoac_client_options_set_credential(opts, cred);
      SUCCEED();
   }
}
