#include <catch2/catch_test_macros.hpp>
#include <mongoac/error.h>
#include <mongoac/tls_options.h>
#include <test_util/error.hh>
#include <test_util/owning_ptr.hh>
#include <test_util/string.hh>

using mongoac::test_util::make_owning_ptr;
using mongoac::test_util::to_mongoac;

TEST_CASE("new", "[mongoac][tls_options]")
{
   SECTION("default")
   {
      auto const opts = make_owning_ptr(mongoac_tls_options_new(), &mongoac_tls_options_destroy);
      REQUIRE(opts != nullptr);
   }
}

TEST_CASE("destroy", "[mongoac][tls_options]")
{
   SECTION("null")
   {
      mongoac_tls_options_destroy(nullptr);
      SUCCEED();
   }
}

TEST_CASE("set_allow_invalid_certificates", "[mongoac][tls_options]")
{
   auto const opts = make_owning_ptr(mongoac_tls_options_new(), &mongoac_tls_options_destroy);

   SECTION("null")
   {
      mongoac_tls_options_set_allow_invalid_certificates(nullptr, true);
      SUCCEED();
   }

   SECTION("set true")
   {
      mongoac_tls_options_set_allow_invalid_certificates(opts, true);
      SUCCEED();
   }

   SECTION("set false")
   {
      mongoac_tls_options_set_allow_invalid_certificates(opts, false);
      SUCCEED();
   }
}

TEST_CASE("set_ca_file_path", "[mongoac][tls_options]")
{
   auto const error = make_owning_ptr(mongoac_error_new(), &mongoac_error_destroy);
   auto const opts = make_owning_ptr(mongoac_tls_options_new(), &mongoac_tls_options_destroy);

   SECTION("null options")
   {
      mongoac_tls_options_set_ca_file_path(nullptr, to_mongoac("/path/to/ca.pem"), error);
      REQUIRE_MONGOAC_INVALID_ARGUMENT(error);
   }

   SECTION("null path clears")
   {
      mongoac_tls_options_set_ca_file_path(opts, {}, error);
      CHECK_MONGOAC_OK(error);
   }

   SECTION("valid")
   {
      mongoac_tls_options_set_ca_file_path(opts, to_mongoac("/path/to/ca.pem"), error);
      CHECK_MONGOAC_OK(error);
   }
}

TEST_CASE("set_cert_key_file_path", "[mongoac][tls_options]")
{
   auto const error = make_owning_ptr(mongoac_error_new(), &mongoac_error_destroy);
   auto const opts = make_owning_ptr(mongoac_tls_options_new(), &mongoac_tls_options_destroy);

   SECTION("null options")
   {
      mongoac_tls_options_set_cert_key_file_path(nullptr, to_mongoac("/path/to/cert.pem"), error);
      REQUIRE_MONGOAC_INVALID_ARGUMENT(error);
   }

   SECTION("null path clears")
   {
      mongoac_tls_options_set_cert_key_file_path(opts, {}, error);
      CHECK_MONGOAC_OK(error);
   }

   SECTION("valid")
   {
      mongoac_tls_options_set_cert_key_file_path(opts, to_mongoac("/path/to/cert.pem"), error);
      CHECK_MONGOAC_OK(error);
   }
}
