#include <mongoac/credential.h>

#include <bson/bson.h>

#include <catch2/catch_test_macros.hpp>
#include <mongoac/error.h>
#include <test_util/bson.hh>
#include <test_util/error.hh>
#include <test_util/owning_ptr.hh>

#include <cstdint>

using mongoac::test_util::bson_from_json;
using mongoac::test_util::make_owning_ptr;

TEST_CASE("new", "[mongoac][credential]")
{
   SECTION("default")
   {
      auto const cred = make_owning_ptr(mongoac_credential_new(), &mongoac_credential_destroy);
      REQUIRE(cred != nullptr);
   }
}

TEST_CASE("destroy", "[mongoac][credential]")
{
   SECTION("null")
   {
      mongoac_credential_destroy(nullptr);
      SUCCEED();
   }
}

TEST_CASE("set_username", "[mongoac][credential]")
{
   auto const error = make_owning_ptr(mongoac_error_new(), &mongoac_error_destroy);
   auto const cred = make_owning_ptr(mongoac_credential_new(), &mongoac_credential_destroy);

   SECTION("null options")
   {
      mongoac_credential_set_username(nullptr, "user", error);
      REQUIRE_MONGOAC_INVALID_ARGUMENT(error);
   }

   SECTION("null string clears")
   {
      mongoac_credential_set_username(cred, nullptr, error);
      CHECK_MONGOAC_OK(error);
   }

   SECTION("valid")
   {
      mongoac_credential_set_username(cred, "user", error);
      CHECK_MONGOAC_OK(error);
   }
}

TEST_CASE("set_source", "[mongoac][credential]")
{
   auto const error = make_owning_ptr(mongoac_error_new(), &mongoac_error_destroy);
   auto const cred = make_owning_ptr(mongoac_credential_new(), &mongoac_credential_destroy);

   SECTION("null string clears")
   {
      mongoac_credential_set_source(cred, nullptr, error);
      CHECK_MONGOAC_OK(error);
   }

   SECTION("valid")
   {
      mongoac_credential_set_source(cred, "admin", error);
      CHECK_MONGOAC_OK(error);
   }
}

TEST_CASE("set_password", "[mongoac][credential]")
{
   auto const error = make_owning_ptr(mongoac_error_new(), &mongoac_error_destroy);
   auto const cred = make_owning_ptr(mongoac_credential_new(), &mongoac_credential_destroy);

   SECTION("null string clears")
   {
      mongoac_credential_set_password(cred, nullptr, error);
      CHECK_MONGOAC_OK(error);
   }

   SECTION("valid")
   {
      mongoac_credential_set_password(cred, "pass", error);
      CHECK_MONGOAC_OK(error);
   }
}

TEST_CASE("set_mechanism", "[mongoac][credential]")
{
   auto const error = make_owning_ptr(mongoac_error_new(), &mongoac_error_destroy);
   auto const cred = make_owning_ptr(mongoac_credential_new(), &mongoac_credential_destroy);

   SECTION("null options")
   {
      mongoac_credential_set_mechanism(nullptr, MONGOAC_AUTH_MECHANISM_SCRAM_SHA_256, error);
      REQUIRE_MONGOAC_INVALID_ARGUMENT(error);
   }

   SECTION("valid scram-sha-256")
   {
      mongoac_credential_set_mechanism(cred, MONGOAC_AUTH_MECHANISM_SCRAM_SHA_256, error);
      CHECK_MONGOAC_OK(error);
   }

   SECTION("valid mongodb-aws")
   {
      mongoac_credential_set_mechanism(cred, MONGOAC_AUTH_MECHANISM_MONGODB_AWS, error);
      CHECK_MONGOAC_OK(error);
   }

   SECTION("invalid mechanism")
   {
      mongoac_credential_set_mechanism(cred, -1, error);
      REQUIRE_MONGOAC_INVALID_ARGUMENT(error);
   }
}

TEST_CASE("set_mechanism_properties", "[mongoac][credential]")
{
   auto const error = make_owning_ptr(mongoac_error_new(), &mongoac_error_destroy);
   auto const cred = make_owning_ptr(mongoac_credential_new(), &mongoac_credential_destroy);

   SECTION("null properties clears")
   {
      mongoac_credential_set_mechanism_properties(cred, {}, error);
      CHECK_MONGOAC_OK(error);
   }

   SECTION("invalid document")
   {
      std::uint8_t data[] = {12, 0, 0, 0, 16, 'x', '\0', 1, 0, 0, 0, 1}; // {"x": 1} with last-byte corruption.

      mongoac_credential_set_mechanism_properties(cred, {data, sizeof(data)}, error);
      CHECK_FALSE_MONGOAC_OK(error);
      CHECK_MONGOAC_ERROR_CATEGORY(error, MONGOAC_ERROR_CATEGORY_BSON);
   }

   SECTION("valid document")
   {
      auto const doc = make_owning_ptr(bson_from_json(R"({"x": 1})"), &bson_destroy);
      mongoac_credential_set_mechanism_properties(cred, make_bson_view(doc), error);
      CHECK_MONGOAC_OK(error);
   }
}
