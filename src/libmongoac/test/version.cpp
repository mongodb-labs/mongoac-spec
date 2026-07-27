#include <mongoac/version.h>

//

#include <catch2/catch_test_macros.hpp>
#include <catch2/matchers/catch_matchers_string.hpp>

TEST_CASE("macros", "[mongoac][version]")
{
   SECTION("consistency")
   {
      CHECK(MONGOAC_VERSION_MAJOR == mongoac_version_major());
      CHECK(MONGOAC_VERSION_MINOR == mongoac_version_minor());
      CHECK(MONGOAC_VERSION_PATCH == mongoac_version_patch());

      CHECK_THAT(mongoac_version(), Catch::Matchers::Equals(MONGOAC_VERSION));
      CHECK_THAT(mongoac_version_prerelease(), Catch::Matchers::Equals(MONGOAC_VERSION_PRERELEASE));
   }

   SECTION("hex")
   {
      CHECK(MONGOAC_VERSION_HEX ==
            (MONGOAC_VERSION_MAJOR << 24 | MONGOAC_VERSION_MINOR << 16 | MONGOAC_VERSION_PATCH << 8));
   }
}

TEST_CASE("MONGOAC_VERSION_CHECK", "[mongoac][version]")
{
   CHECK(MONGOAC_VERSION_CHECK(0, 0, 0));
   CHECK(MONGOAC_VERSION_CHECK(MONGOAC_VERSION_MAJOR, MONGOAC_VERSION_MINOR, MONGOAC_VERSION_PATCH));

   CHECK(!MONGOAC_VERSION_CHECK(MONGOAC_VERSION_MAJOR + 1, MONGOAC_VERSION_MINOR, MONGOAC_VERSION_PATCH));
   CHECK(!MONGOAC_VERSION_CHECK(MONGOAC_VERSION_MAJOR, MONGOAC_VERSION_MINOR + 1, MONGOAC_VERSION_PATCH));
   CHECK(!MONGOAC_VERSION_CHECK(MONGOAC_VERSION_MAJOR, MONGOAC_VERSION_MINOR, MONGOAC_VERSION_PATCH + 1));
}

TEST_CASE("check", "[mongoac][version]")
{
   const int major = mongoac_version_major();
   const int minor = mongoac_version_minor();
   const int patch = mongoac_version_patch();

   SECTION("0.0.0")
   {
      CHECK(mongoac_version_check(0, 0, 0));
   }

   SECTION("exact match")
   {
      CHECK(mongoac_version_check(major, minor, patch));
   }

   SECTION("newer versions")
   {
      CHECK(!mongoac_version_check(major + 1, minor, patch));
      CHECK(!mongoac_version_check(major, minor + 1, patch));
      CHECK(!mongoac_version_check(major, minor, patch + 1));
   }
}
