#pragma once

#include <bson/bson_t.h>

#include <catch2/catch_test_macros.hpp>

#include <cstring>

namespace mongoac
{
namespace test_util
{

bool
bson_array_contains_string(const bson_t *array, const char *str);

} // namespace test_util
} // namespace mongoac
