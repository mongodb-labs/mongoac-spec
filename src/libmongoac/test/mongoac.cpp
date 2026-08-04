#include <catch2/catch_test_macros.hpp>
#include <mongoac/sanity_check.h>

TEST_CASE("sanity_check", "[mongoac]")
{
   CHECK(mongoac_sanity_check(1) == 1);
}
