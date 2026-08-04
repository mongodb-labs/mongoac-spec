#pragma once

#include <bson/bson_t.h>

#include <catch2/catch_test_macros.hpp>

#include <cstring>

namespace mongoac
{
namespace test_util
{

bson_t *
bson_from_json(const char *json);

bool
bson_array_contains_string(const bson_t *array, const char *str);

} // namespace test_util
} // namespace mongoac
