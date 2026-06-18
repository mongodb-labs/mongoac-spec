#include <mongoac/sanity_check.h>

#include <catch2/catch_test_macros.hpp>

TEST_CASE("sanity_check", "[mongoac]") {
  CHECK(mongoac_sanity_check(1) == 1);
}
