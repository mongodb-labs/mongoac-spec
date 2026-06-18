:man_page: mongoac_version

mongoac_version
===============

Synopsis
--------

.. code-block:: c

   #define MONGOAC_VERSION            // e.g. "1.2.3-dev"
   #define MONGOAC_VERSION_MAJOR      // e.g. 1
   #define MONGOAC_VERSION_MINOR      // e.g. 2
   #define MONGOAC_VERSION_PATCH      // e.g. 3
   #define MONGOAC_VERSION_PRERELEASE // e.g. "dev" or ""

   #define MONGOAC_VERSION_HEX // e.g. 0x01020300

   #define MONGOAC_VERSION_CHECK(major, minor, patch)

   const char *mongoac_version(void); // Runtime equivalent to MONGOAC_VERSION.

   int32_t mongoac_version_major(void); // Runtime equivalent to MONGOAC_VERSION_MAJOR.
   int32_t mongoac_version_minor(void); // Runtime equivalent to MONGOAC_VERSION_MINOR.
   int32_t mongoac_version_patch(void); // Runtime equivalent to MONGOAC_VERSION_PATCH.
   const char *mongoac_version_prerelease(void); // Runtime equivalent to MONGOAC_VERSION_PRERELEASE.

   // Runtime equivalent to MONGOAC_VERSION_CHECK.
   bool mongoac_version_check(int32_t required_major, int32_t required_minor, int32_t required_patch);

Description
-----------

Describes the mongoac library version.
