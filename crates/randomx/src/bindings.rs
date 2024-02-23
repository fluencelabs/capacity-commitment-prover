/* Generated with help of rust-bindgen 0.69.2 */

#![allow(dead_code)]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]

pub mod flags {
    pub const randomx_flags_RANDOMX_FLAG_DEFAULT: randomx_flags = 0;
    pub const randomx_flags_RANDOMX_FLAG_LARGE_PAGES: randomx_flags = 1;
    pub const randomx_flags_RANDOMX_FLAG_HARD_AES: randomx_flags = 2;
    pub const randomx_flags_RANDOMX_FLAG_FULL_MEM: randomx_flags = 4;
    pub const randomx_flags_RANDOMX_FLAG_JIT: randomx_flags = 8;
    pub const randomx_flags_RANDOMX_FLAG_SECURE: randomx_flags = 16;
    pub const randomx_flags_RANDOMX_FLAG_ARGON2_SSSE3: randomx_flags = 32;
    pub const randomx_flags_RANDOMX_FLAG_ARGON2_AVX2: randomx_flags = 64;
    pub const randomx_flags_RANDOMX_FLAG_ARGON2: randomx_flags = 96;

    pub type randomx_flags = ::std::os::raw::c_uint;

    extern "C" {
        #[doc = " @return The recommended flags to be used on the current machine.
                 Does not include:
                    - RANDOMX_FLAG_LARGE_PAGES
                    - RANDOMX_FLAG_FULL_MEM
                    - RANDOMX_FLAG_SECURE
                 These flags must be added manually if desired.
                 On OpenBSD RANDOMX_FLAG_SECURE is enabled by default in JIT mode as W^X is enforced by the OS.
        "]
        pub fn randomx_get_flags() -> randomx_flags;
    }
}

pub mod cache {
    use super::flags::randomx_flags;

    #[repr(C)]
    #[derive(Debug, Copy, Clone)]
    pub struct randomx_cache {
        _unused: [u8; 0],
    }

    extern "C" {
        #[doc = " Creates a randomx_cache structure and allocates memory for RandomX Cache.

         @param flags is any combination of these 2 flags (each flag can be set or not set):
                 RANDOMX_FLAG_LARGE_PAGES - allocate memory in large pages
                 RANDOMX_FLAG_JIT - create cache structure with JIT compilation support; this makes
                                    subsequent Dataset initialization faster
         Optionally, one of these two flags may be selected:
                 RANDOMX_FLAG_ARGON2_SSSE3 - optimized Argon2 for CPUs with the SSSE3 instruction set
                                             makes subsequent cache initialization faster
                 RANDOMX_FLAG_ARGON2_AVX2 - optimized Argon2 for CPUs with the AVX2 instruction set
                                            makes subsequent cache initialization faster

         @return Pointer to an allocated randomx_cache structure.
                 Returns NULL if:
                          (1) memory allocation fails
                          (2) the RANDOMX_FLAG_JIT is set and JIT compilation is not supported on the current platform
                          (3) an invalid or unsupported RANDOMX_FLAG_ARGON2 value is set
        "]
        pub fn randomx_alloc_cache(flags: randomx_flags) -> *mut randomx_cache;
    }

    extern "C" {
        #[doc = " Initializes the cache memory and SuperscalarHash using the provided key value.
          Does nothing if called again with the same key value.

          @param cache is a pointer to a previously allocated randomx_cache structure. Must not be NULL.
          @param key is a pointer to memory which contains the key value. Must not be NULL.
          @param keySize is the number of bytes of the key.
        "]
        pub fn randomx_init_cache(
            cache: *mut randomx_cache,
            key: *const ::std::os::raw::c_void,
            keySize: usize,
        );
    }

    extern "C" {
        #[doc = " Releases all memory occupied by the randomx_cache structure.

          @param cache is a pointer to a previously allocated randomx_cache structure.
        "]
        pub fn randomx_release_cache(cache: *mut randomx_cache);
    }
}

pub mod dataset {
    use super::cache::randomx_cache;
    use super::flags::randomx_flags;

    #[repr(C)]
    #[derive(Debug, Copy, Clone)]
    pub struct randomx_dataset {
        _unused: [u8; 0],
    }

    extern "C" {
        #[doc = " Creates a randomx_dataset structure and allocates memory for RandomX Dataset.

           @param flags is the initialization flags. Only one flag is supported (can be set or not set):
                   RANDOMX_FLAG_LARGE_PAGES - allocate memory in large pages

           @return Pointer to an allocated randomx_dataset structure.
                    NULL is returned if memory allocation fails.
        "]
        pub fn randomx_alloc_dataset(flags: randomx_flags) -> *mut randomx_dataset;
    }

    extern "C" {
        #[doc = " Gets the number of items contained in the dataset.

          @return the number of items contained in the dataset.
        "]
        pub fn randomx_dataset_item_count() -> ::std::os::raw::c_ulong;
    }

    extern "C" {
        #[doc = " Initializes dataset items.

          Note: In order to use the Dataset, all items from 0 to (randomx_dataset_item_count() - 1) must be initialized.
          This may be done by several calls to this function using non-overlapping item sequences.

          @param dataset is a pointer to a previously allocated randomx_dataset structure. Must not be NULL.
          @param cache is a pointer to a previously allocated and initialized randomx_cache structure. Must not be NULL.
          @param startItem is the item number where intialization should start.
          @param itemCount is the number of items that should be initialized.
        "]
        pub fn randomx_init_dataset(
            dataset: *mut randomx_dataset,
            cache: *mut randomx_cache,
            startItem: ::std::os::raw::c_ulong,
            itemCount: ::std::os::raw::c_ulong,
        );
    }

    extern "C" {
        #[doc = " Returns a pointer to the internal memory buffer of the dataset structure. The size
           of the internal memory buffer is randomx_dataset_item_count() * RANDOMX_DATASET_ITEM_SIZE.

           @param dataset is a pointer to a previously allocated randomx_dataset structure. Must not be NULL.
           @return Pointer to the internal memory buffer of the dataset structure.
        "]
        pub fn randomx_get_dataset_memory(
            dataset: *mut randomx_dataset,
        ) -> *mut ::std::os::raw::c_void;
    }

    extern "C" {
        #[doc = " Releases all memory occupied by the randomx_dataset structure.\
          @param dataset is a pointer to a previously allocated randomx_dataset structure.\
        "]
        pub fn randomx_release_dataset(dataset: *mut randomx_dataset);
    }
}

pub mod vm {
    use super::cache::randomx_cache;
    use super::dataset::randomx_dataset;
    use super::flags::randomx_flags;

    #[repr(C)]
    #[derive(Debug, Copy, Clone)]
    pub struct randomx_vm {
        _unused: [u8; 0],
    }

    extern "C" {
        #[doc = " Creates and initializes a RandomX virtual machine.

          @param flags is any combination of these 5 flags (each flag can be set or not set):
                  RANDOMX_FLAG_LARGE_PAGES - allocate scratchpad memory in large pages
                  RANDOMX_FLAG_HARD_AES - virtual machine will use hardware accelerated AES
                  RANDOMX_FLAG_FULL_MEM - virtual machine will use the full dataset
                  RANDOMX_FLAG_JIT - virtual machine will use a JIT compiler
                  RANDOMX_FLAG_SECURE - when combined with RANDOMX_FLAG_JIT, the JIT pages are never
                                        writable and executable at the same time (W^X policy)
                  The numeric values of the first 4 flags are ordered so that a higher value will provide
                  faster hash calculation and a lower numeric value will provide higher portability.
                  Using RANDOMX_FLAG_DEFAULT (all flags not set) works on all platforms, but is the slowest.
          @param cache is a pointer to an initialized randomx_cache structure. Can be
                  NULL if RANDOMX_FLAG_FULL_MEM is set.
          @param dataset is a pointer to a randomx_dataset structure. Can be NULL
                  if RANDOMX_FLAG_FULL_MEM is not set.

          @return Pointer to an initialized randomx_vm structure.
                  Returns NULL if:
                  (1) Scratchpad memory allocation fails.
                  (2) The requested initialization flags are not supported on the current platform.
                  (3) cache parameter is NULL and RANDOMX_FLAG_FULL_MEM is not set
                  (4) dataset parameter is NULL and RANDOMX_FLAG_FULL_MEM is set
         "]
        pub fn randomx_create_vm(
            flags: randomx_flags,
            cache: *mut randomx_cache,
            dataset: *mut randomx_dataset,
        ) -> *mut randomx_vm;
    }

    extern "C" {
        #[doc = " Reinitializes a virtual machine with a new Cache. This function should be called anytime
          the Cache is reinitialized with a new key. Does nothing if called with a Cache containing
          the same key value as already set.

           @param machine is a pointer to a randomx_vm structure that was initialized
                  without RANDOMX_FLAG_FULL_MEM. Must not be NULL.
           @param cache is a pointer to an initialized randomx_cache structure. Must not be NULL.
        "]
        pub fn randomx_vm_set_cache(machine: *mut randomx_vm, cache: *mut randomx_cache);
    }

    extern "C" {
        #[doc = " Reinitializes a virtual machine with a new Dataset.
          @param machine is a pointer to a randomx_vm structure that was initialized
                 with RANDOMX_FLAG_FULL_MEM. Must not be NULL.
          @param dataset is a pointer to an initialized randomx_dataset structure. Must not be NULL.
        "]
        pub fn randomx_vm_set_dataset(machine: *mut randomx_vm, dataset: *mut randomx_dataset);
    }

    extern "C" {
        #[doc = " Releases all memory occupied by the randomx_vm structure.
          @param machine is a pointer to a previously created randomx_vm structure.\
        "]
        pub fn randomx_destroy_vm(machine: *mut randomx_vm);
    }

    extern "C" {
        #[doc = " Calculates a RandomX hash value.
          @param machine is a pointer to a randomx_vm structure. Must not be NULL.
          @param input is a pointer to memory to be hashed. Must not be NULL.
          @param inputSize is the number of bytes to be hashed.
          @param output is a pointer to memory where the hash will be stored. Must not
                 be NULL and at least RANDOMX_HASH_SIZE bytes must be available for writing.
        "]
        pub fn randomx_calculate_hash(
            machine: *mut randomx_vm,
            input: *const ::std::os::raw::c_void,
            inputSize: usize,
            output: *mut ::std::os::raw::c_void,
        );
    }

    extern "C" {
        #[doc = " Set of functions used to calculate multiple RandomX hashes more efficiently.
           randomx_calculate_hash_first will begin a hash calculation.
           randomx_calculate_hash_next  will output the hash value of the previous input
                                        and begin the calculation of the next hash.
           randomx_calculate_hash_last  will output the hash value of the previous input.
           WARNING: These functions may alter the floating point rounding mode of the calling thread.

           @param machine is a pointer to a randomx_vm structure. Must not be NULL.
           @param input is a pointer to memory to be hashed. Must not be NULL.
           @param inputSize is the number of bytes to be hashed.
           @param nextInput is a pointer to memory to be hashed for the next hash. Must not be NULL.
           @param nextInputSize is the number of bytes to be hashed for the next hash.
           @param output is a pointer to memory where the hash will be stored. Must not
                  be NULL and at least RANDOMX_HASH_SIZE bytes must be available for writing.
        "]
        pub fn randomx_calculate_hash_first(
            machine: *mut randomx_vm,
            input: *const ::std::os::raw::c_void,
            inputSize: usize,
        );
    }

    extern "C" {
        pub fn randomx_calculate_hash_next(
            machine: *mut randomx_vm,
            nextInput: *const ::std::os::raw::c_void,
            nextInputSize: usize,
            output: *mut ::std::os::raw::c_void,
        );
    }

    extern "C" {
        pub fn randomx_calculate_hash_last(
            machine: *mut randomx_vm,
            output: *mut ::std::os::raw::c_void,
        );
    }
}
