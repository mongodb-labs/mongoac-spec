#include <common-thread-private.h>

#include <mongoc/mongoc.h>

#include <bson/bson.h>

#include <TestSuite.h>
#include <test-conveniences.h>
#include <test-libmongoc.h>

int
main(int argc, char *argv[])
{
   TestSuite suite;

   test_libmongoc_init(&suite, argc, argv);

   // Snipped.

   const int ret = TestSuite_Run(&suite);

   test_libmongoc_destroy(&suite);

   return ret;
}
