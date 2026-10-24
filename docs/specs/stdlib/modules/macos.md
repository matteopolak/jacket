# macOS

## Public declaration inventory

Visible public declarations and module-conditional variants are listed below. Procedure bodies and aggregate initializers are omitted; complete public type members are listed separately.

### macOS-specific declarations

- `kCFStringEncodingInvalidId :: 0xffffffff`
- `__kCFStringInlineBufferLength :: 64`
- `CFTypeID :: u64`
- `CFOptionFlags :: u64`
- `CFHashCode :: u64`
- `CFIndex :: s64`
- `CFTypeRef :: *void`
- `__CFString :: struct`
- `CFStringRef :: *__CFString`
- `CFMutableStringRef :: *__CFString`
- `CFPropertyListRef :: CFTypeRef`
- `CFComparisonResult :: enum CFIndex`
- `CFComparatorFunction :: #type (val1: *void, val2: *void, _context: *void) -> CFComparisonResult #c_call`
- `kCFNotFound : CFIndex`
- `CFRange :: struct`
- `__CFRangeMake :: (loc: CFIndex, len: CFIndex) -> CFRange #foreign corefoundation`
- `__CFNull :: struct`
- `CFNullRef :: *__CFNull`
- `CFNullGetTypeID :: () -> CFTypeID #foreign corefoundation`
- `__CFAllocator :: struct`
- `CFAllocatorRef :: *__CFAllocator`
- `CFAllocatorRetainCallBack :: #type (info: *void) -> *void #c_call`
- `CFAllocatorReleaseCallBack :: #type (info: *void) -> void #c_call`
- `CFAllocatorCopyDescriptionCallBack :: #type (info: *void) -> CFStringRef #c_call`
- `CFAllocatorAllocateCallBack :: #type (allocSize: CFIndex, hint: CFOptionFlags, info: *void) -> *void #c_call`
- `CFAllocatorReallocateCallBack :: #type (ptr: *void, newsize: CFIndex, hint: CFOptionFlags, info: *void) -> *void #c_call`
- `CFAllocatorDeallocateCallBack :: #type (ptr: *void, info: *void) -> void #c_call`
- `CFAllocatorPreferredSizeCallBack :: #type (size: CFIndex, hint: CFOptionFlags, info: *void) -> CFIndex #c_call`
- `CFAllocatorContext :: struct`
- `CFAllocatorGetTypeID :: () -> CFTypeID #foreign corefoundation`
- `CFAllocatorSetDefault :: (allocator: CFAllocatorRef) -> void #foreign corefoundation`
- `CFAllocatorGetDefault :: () -> CFAllocatorRef #foreign corefoundation`
- `CFAllocatorCreate :: (allocator: CFAllocatorRef, _context: *CFAllocatorContext) -> CFAllocatorRef #foreign corefoundation`
- `CFAllocatorAllocate :: (allocator: CFAllocatorRef, size: CFIndex, hint: CFOptionFlags) -> *void #foreign corefoundation`
- `CFAllocatorReallocate :: (allocator: CFAllocatorRef, ptr: *void, newsize: CFIndex, hint: CFOptionFlags) -> *void #foreign corefoundation`
- `CFAllocatorDeallocate :: (allocator: CFAllocatorRef, ptr: *void) -> void #foreign corefoundation`
- `CFAllocatorGetPreferredSizeForSize :: (allocator: CFAllocatorRef, size: CFIndex, hint: CFOptionFlags) -> CFIndex #foreign corefoundation`
- `CFAllocatorGetContext :: (allocator: CFAllocatorRef, _context: *CFAllocatorContext) -> void #foreign corefoundation`
- `CFGetTypeID :: (cf: CFPropertyListRef) -> CFTypeID #foreign corefoundation`
- `CFCopyTypeIDDescription :: (type_id: CFTypeID) -> CFStringRef #foreign corefoundation`
- `CFRetain :: (cf: CFPropertyListRef) -> CFPropertyListRef #foreign corefoundation`
- `CFRelease :: (cf: CFPropertyListRef) -> void #foreign corefoundation`
- `CFAutorelease :: (arg: CFPropertyListRef) -> CFPropertyListRef #foreign corefoundation`
- `CFGetRetainCount :: (cf: CFPropertyListRef) -> CFIndex #foreign corefoundation`
- `CFEqual :: (cf1: CFPropertyListRef, cf2: CFPropertyListRef) -> Boolean #foreign corefoundation`
- `CFHash :: (cf: CFPropertyListRef) -> CFHashCode #foreign corefoundation`
- `CFCopyDescription :: (cf: CFPropertyListRef) -> CFStringRef #foreign corefoundation`
- `CFGetAllocator :: (cf: CFPropertyListRef) -> CFAllocatorRef #foreign corefoundation`
- `CFMakeCollectable :: (cf: CFPropertyListRef) -> CFPropertyListRef #foreign corefoundation`
- `CFArrayRetainCallBack :: #type (allocator: CFAllocatorRef, value: *void) -> *void #c_call`
- `CFArrayReleaseCallBack :: #type (allocator: CFAllocatorRef, value: *void) -> void #c_call`
- `CFArrayCopyDescriptionCallBack :: #type (value: *void) -> CFStringRef #c_call`
- `CFArrayEqualCallBack :: #type (value1: *void, value2: *void) -> Boolean #c_call`
- `CFArrayCallBacks :: struct`
- `CFArrayApplierFunction :: #type (value: *void, _context: *void) -> void #c_call`
- `__CFArray :: struct`
- `CFArrayRef :: *__CFArray`
- `CFMutableArrayRef :: *__CFArray`
- `CFArrayGetTypeID :: () -> CFTypeID #foreign corefoundation`
- `CFArrayCreate :: (allocator: CFAllocatorRef, values: **void, numValues: CFIndex, callBacks: *CFArrayCallBacks) -> CFArrayRef #foreign corefoundation`
- `CFArrayCreateCopy :: (allocator: CFAllocatorRef, theArray: CFArrayRef) -> CFArrayRef #foreign corefoundation`
- `CFArrayCreateMutable :: (allocator: CFAllocatorRef, capacity: CFIndex, callBacks: *CFArrayCallBacks) -> CFMutableArrayRef #foreign corefoundation`
- `CFArrayCreateMutableCopy :: (allocator: CFAllocatorRef, capacity: CFIndex, theArray: CFArrayRef) -> CFMutableArrayRef #foreign corefoundation`
- `CFArrayGetCount :: (theArray: CFArrayRef) -> CFIndex #foreign corefoundation`
- `CFArrayGetCountOfValue :: (theArray: CFArrayRef, range: CFRange, value: *void) -> CFIndex #foreign corefoundation`
- `CFArrayContainsValue :: (theArray: CFArrayRef, range: CFRange, value: *void) -> Boolean #foreign corefoundation`
- `CFArrayGetValueAtIndex :: (theArray: CFArrayRef, idx: CFIndex) -> *void #foreign corefoundation`
- `CFArrayGetValues :: (theArray: CFArrayRef, range: CFRange, values: **void) -> void #foreign corefoundation`
- `CFArrayApplyFunction :: (theArray: CFArrayRef, range: CFRange, applier: CFArrayApplierFunction, _context: *void) -> void #foreign corefoundation`
- `CFArrayGetFirstIndexOfValue :: (theArray: CFArrayRef, range: CFRange, value: *void) -> CFIndex #foreign corefoundation`
- `CFArrayGetLastIndexOfValue :: (theArray: CFArrayRef, range: CFRange, value: *void) -> CFIndex #foreign corefoundation`
- `CFArrayBSearchValues :: (theArray: CFArrayRef, range: CFRange, value: *void, comparator: CFComparatorFunction, _context: *void) -> CFIndex #foreign corefoundation`
- `CFArrayAppendValue :: (theArray: CFMutableArrayRef, value: *void) -> void #foreign corefoundation`
- `CFArrayInsertValueAtIndex :: (theArray: CFMutableArrayRef, idx: CFIndex, value: *void) -> void #foreign corefoundation`
- `CFArraySetValueAtIndex :: (theArray: CFMutableArrayRef, idx: CFIndex, value: *void) -> void #foreign corefoundation`
- `CFArrayRemoveValueAtIndex :: (theArray: CFMutableArrayRef, idx: CFIndex) -> void #foreign corefoundation`
- `CFArrayRemoveAllValues :: (theArray: CFMutableArrayRef) -> void #foreign corefoundation`
- `CFArrayReplaceValues :: (theArray: CFMutableArrayRef, range: CFRange, newValues: **void, newCount: CFIndex) -> void #foreign corefoundation`
- `CFArrayExchangeValuesAtIndices :: (theArray: CFMutableArrayRef, idx1: CFIndex, idx2: CFIndex) -> void #foreign corefoundation`
- `CFArraySortValues :: (theArray: CFMutableArrayRef, range: CFRange, comparator: CFComparatorFunction, _context: *void) -> void #foreign corefoundation`
- `CFArrayAppendArray :: (theArray: CFMutableArrayRef, otherArray: CFArrayRef, otherRange: CFRange) -> void #foreign corefoundation`
- `__CFBag :: struct`
- `CFMutableBagRef :: *__CFBag`
- `__CFBinaryHeap :: struct`
- `CFBit :: u32`
- `__CFBitVector :: struct`
- `CFMutableBitVectorRef :: *__CFBitVector`
- `__CFByteOrder :: enum u32`
- `CFByteOrder :: CFIndex`
- `CFSwappedFloat32 :: struct`
- `CFSwappedFloat64 :: struct`
- `CFDictionaryRetainCallBack :: #type (allocator: CFAllocatorRef, value: *void) -> *void #c_call`
- `CFDictionaryReleaseCallBack :: #type (allocator: CFAllocatorRef, value: *void) -> void #c_call`
- `CFDictionaryCopyDescriptionCallBack :: #type (value: *void) -> CFStringRef #c_call`
- `CFDictionaryEqualCallBack :: #type (value1: *void, value2: *void) -> Boolean #c_call`
- `CFDictionaryHashCallBack :: #type (value: *void) -> CFHashCode #c_call`
- `CFDictionaryKeyCallBacks :: struct`
- `CFDictionaryValueCallBacks :: struct`
- `CFDictionaryApplierFunction :: #type (key: *void, value: *void, _context: *void) -> void #c_call`
- `__CFDictionary :: struct`
- `CFDictionaryRef :: *__CFDictionary`
- `CFMutableDictionaryRef :: *__CFDictionary`
- `CFDictionaryGetTypeID :: () -> CFTypeID #foreign corefoundation`
- `CFDictionaryCreate :: (allocator: CFAllocatorRef, keys: **void, values: **void, numValues: CFIndex, keyCallBacks: *CFDictionaryKeyCallBacks, valueCallBacks: *CFDictionaryValueCallBacks) -> CFDictionaryRef #foreign corefoundation`
- `CFDictionaryCreateCopy :: (allocator: CFAllocatorRef, theDict: CFDictionaryRef) -> CFDictionaryRef #foreign corefoundation`
- `CFDictionaryCreateMutable :: (allocator: CFAllocatorRef, capacity: CFIndex, keyCallBacks: *CFDictionaryKeyCallBacks, valueCallBacks: *CFDictionaryValueCallBacks) -> CFMutableDictionaryRef #foreign corefoundation`
- `CFDictionaryCreateMutableCopy :: (allocator: CFAllocatorRef, capacity: CFIndex, theDict: CFDictionaryRef) -> CFMutableDictionaryRef #foreign corefoundation`
- `CFDictionaryGetCount :: (theDict: CFDictionaryRef) -> CFIndex #foreign corefoundation`
- `CFDictionaryGetCountOfKey :: (theDict: CFDictionaryRef, key: *void) -> CFIndex #foreign corefoundation`
- `CFDictionaryGetCountOfValue :: (theDict: CFDictionaryRef, value: *void) -> CFIndex #foreign corefoundation`
- `CFDictionaryContainsKey :: (theDict: CFDictionaryRef, key: *void) -> Boolean #foreign corefoundation`
- `CFDictionaryContainsValue :: (theDict: CFDictionaryRef, value: *void) -> Boolean #foreign corefoundation`
- `CFDictionaryGetValue :: (theDict: CFDictionaryRef, key: *void) -> *void #foreign corefoundation`
- `CFDictionaryGetValueIfPresent :: (theDict: CFDictionaryRef, key: *void, value: **void) -> Boolean #foreign corefoundation`
- `CFDictionaryGetKeysAndValues :: (theDict: CFDictionaryRef, keys: **void, values: **void) -> void #foreign corefoundation`
- `CFDictionaryApplyFunction :: (theDict: CFDictionaryRef, applier: CFDictionaryApplierFunction, _context: *void) -> void #foreign corefoundation`
- `CFDictionaryAddValue :: (theDict: CFMutableDictionaryRef, key: *void, value: *void) -> void #foreign corefoundation`
- `CFDictionarySetValue :: (theDict: CFMutableDictionaryRef, key: *void, value: *void) -> void #foreign corefoundation`
- `CFDictionaryReplaceValue :: (theDict: CFMutableDictionaryRef, key: *void, value: *void) -> void #foreign corefoundation`
- `CFDictionaryRemoveValue :: (theDict: CFMutableDictionaryRef, key: *void) -> void #foreign corefoundation`
- `CFDictionaryRemoveAllValues :: (theDict: CFMutableDictionaryRef) -> void #foreign corefoundation`
- `CFNotificationName :: CFStringRef`
- `__CFNotificationCenter :: struct`
- `CFNotificationCenterRef :: *__CFNotificationCenter`
- `CFNotificationCallback :: #type (center: CFNotificationCenterRef, observer: *void, name: CFNotificationName, object: *void, userInfo: CFDictionaryRef) -> void #c_call`
- `CFNotificationSuspensionBehavior :: enum CFIndex`
- `CFNotificationCenterGetTypeID :: () -> CFTypeID #foreign corefoundation`
- `CFNotificationCenterGetLocalCenter :: () -> CFNotificationCenterRef #foreign corefoundation`
- `CFNotificationCenterGetDistributedCenter :: () -> CFNotificationCenterRef #foreign corefoundation`
- `CFNotificationCenterGetDarwinNotifyCenter :: () -> CFNotificationCenterRef #foreign corefoundation`
- `CFNotificationCenterAddObserver :: (center: CFNotificationCenterRef, observer: *void, callBack: CFNotificationCallback, name: CFStringRef, object: *void, suspensionBehavior: CFNotificationSuspensionBehavior) -> void #foreign corefoundation`
- `CFNotificationCenterRemoveObserver :: (center: CFNotificationCenterRef, observer: *void, name: CFNotificationName, object: *void) -> void #foreign corefoundation`
- `CFNotificationCenterRemoveEveryObserver :: (center: CFNotificationCenterRef, observer: *void) -> void #foreign corefoundation`
- `CFNotificationCenterPostNotification :: (center: CFNotificationCenterRef, name: CFNotificationName, object: *void, userInfo: CFDictionaryRef, deliverImmediately: Boolean) -> void #foreign corefoundation`
- `kCFNotification :: enum u32`
- `CFNotificationCenterPostNotificationWithOptions :: (center: CFNotificationCenterRef, name: CFNotificationName, object: *void, userInfo: CFDictionaryRef, options: CFOptionFlags) -> void #foreign corefoundation`
- `CFLocaleIdentifier :: CFStringRef`
- `CFLocaleKey :: CFStringRef`
- `__CFLocale :: struct`
- `CFLocaleRef :: *__CFLocale`
- `CFLocaleGetTypeID :: () -> CFTypeID #foreign corefoundation`
- `CFLocaleGetSystem :: () -> CFLocaleRef #foreign corefoundation`
- `CFLocaleCopyCurrent :: () -> CFLocaleRef #foreign corefoundation`
- `CFLocaleCopyAvailableLocaleIdentifiers :: () -> CFArrayRef #foreign corefoundation`
- `CFLocaleCopyISOLanguageCodes :: () -> CFArrayRef #foreign corefoundation`
- `CFLocaleCopyISOCountryCodes :: () -> CFArrayRef #foreign corefoundation`
- `CFLocaleCopyISOCurrencyCodes :: () -> CFArrayRef #foreign corefoundation`
- `CFLocaleCopyCommonISOCurrencyCodes :: () -> CFArrayRef #foreign corefoundation`
- `CFLocaleCopyPreferredLanguages :: () -> CFArrayRef #foreign corefoundation`
- `CFLocaleCreateCanonicalLanguageIdentifierFromString :: (allocator: CFAllocatorRef, localeIdentifier: CFStringRef) -> CFLocaleIdentifier #foreign corefoundation`
- `CFLocaleCreateCanonicalLocaleIdentifierFromString :: (allocator: CFAllocatorRef, localeIdentifier: CFStringRef) -> CFLocaleIdentifier #foreign corefoundation`
- `CFLocaleCreateCanonicalLocaleIdentifierFromScriptManagerCodes :: (allocator: CFAllocatorRef, lcode: LangCode, rcode: RegionCode) -> CFLocaleIdentifier #foreign corefoundation`
- `CFLocaleCreateLocaleIdentifierFromWindowsLocaleCode :: (allocator: CFAllocatorRef, lcid: u32) -> CFLocaleIdentifier #foreign corefoundation`
- `CFLocaleGetWindowsLocaleCodeFromLocaleIdentifier :: (localeIdentifier: CFLocaleIdentifier) -> u32 #foreign corefoundation`
- `CFLocaleLanguageDirection :: enum CFIndex`
- `CFLocaleGetLanguageCharacterDirection :: (isoLangCode: CFStringRef) -> CFLocaleLanguageDirection #foreign corefoundation`
- `CFLocaleGetLanguageLineDirection :: (isoLangCode: CFStringRef) -> CFLocaleLanguageDirection #foreign corefoundation`
- `CFLocaleCreateComponentsFromLocaleIdentifier :: (allocator: CFAllocatorRef, localeID: CFLocaleIdentifier) -> CFDictionaryRef #foreign corefoundation`
- `CFLocaleCreateLocaleIdentifierFromComponents :: (allocator: CFAllocatorRef, dictionary: CFDictionaryRef) -> CFLocaleIdentifier #foreign corefoundation`
- `CFLocaleCreate :: (allocator: CFAllocatorRef, localeIdentifier: CFLocaleIdentifier) -> CFLocaleRef #foreign corefoundation`
- `CFLocaleCreateCopy :: (allocator: CFAllocatorRef, locale: CFLocaleRef) -> CFLocaleRef #foreign corefoundation`
- `CFLocaleGetIdentifier :: (locale: CFLocaleRef) -> CFLocaleIdentifier #foreign corefoundation`
- `CFLocaleGetValue :: (locale: CFLocaleRef, key: CFLocaleKey) -> CFPropertyListRef #foreign corefoundation`
- `CFLocaleCopyDisplayNameForPropertyValue :: (displayLocale: CFLocaleRef, key: CFLocaleKey, value: CFStringRef) -> CFStringRef #foreign corefoundation`
- `CFCalendarIdentifier :: CFStringRef`
- `CFTimeInterval :: float64`
- `CFAbsoluteTime :: CFTimeInterval`
- `CFAbsoluteTimeGetCurrent :: () -> CFAbsoluteTime #foreign corefoundation`
- `__CFDate :: struct`
- `CFDateRef :: *__CFDate`
- `CFDateGetTypeID :: () -> CFTypeID #foreign corefoundation`
- `CFDateCreate :: (allocator: CFAllocatorRef, at: CFAbsoluteTime) -> CFDateRef #foreign corefoundation`
- `CFDateGetAbsoluteTime :: (theDate: CFDateRef) -> CFAbsoluteTime #foreign corefoundation`
- `CFDateGetTimeIntervalSinceDate :: (theDate: CFDateRef, otherDate: CFDateRef) -> CFTimeInterval #foreign corefoundation`
- `CFDateCompare :: (theDate: CFDateRef, otherDate: CFDateRef, _context: *void) -> CFComparisonResult #foreign corefoundation`
- `__CFTimeZone :: struct`
- `CFTimeZoneRef :: *__CFTimeZone`
- `CFGregorianDate :: struct`
- `CFGregorianUnits :: struct`
- `CFGregorianUnitFlags :: CFOptionFlags`
- `kCFGregorian :: enum u32`
- `CFGregorianDateIsValid :: (gdate: CFGregorianDate, unitFlags: CFOptionFlags) -> Boolean #foreign corefoundation`
- `CFGregorianDateGetAbsoluteTime :: (gdate: CFGregorianDate, tz: CFTimeZoneRef) -> CFAbsoluteTime #foreign corefoundation`
- `CFAbsoluteTimeGetGregorianDate :: (at: CFAbsoluteTime, tz: CFTimeZoneRef) -> CFGregorianDate #foreign corefoundation`
- `CFAbsoluteTimeAddGregorianUnits :: (at: CFAbsoluteTime, tz: CFTimeZoneRef, units: CFGregorianUnits) -> CFAbsoluteTime #foreign corefoundation`
- `CFAbsoluteTimeGetDifferenceAsGregorianUnits :: (at1: CFAbsoluteTime, at2: CFAbsoluteTime, tz: CFTimeZoneRef, unitFlags: CFOptionFlags) -> CFGregorianUnits #foreign corefoundation`
- `CFAbsoluteTimeGetDayOfWeek :: (at: CFAbsoluteTime, tz: CFTimeZoneRef) -> s32 #foreign corefoundation`
- `CFAbsoluteTimeGetDayOfYear :: (at: CFAbsoluteTime, tz: CFTimeZoneRef) -> s32 #foreign corefoundation`
- `CFAbsoluteTimeGetWeekOfYear :: (at: CFAbsoluteTime, tz: CFTimeZoneRef) -> s32 #foreign corefoundation`
- `__CFData :: struct`
- `CFDataRef :: *__CFData`
- `CFMutableDataRef :: *__CFData`
- `CFDataGetTypeID :: () -> CFTypeID #foreign corefoundation`
- `CFDataCreate :: (allocator: CFAllocatorRef, bytes: *u8, length: CFIndex) -> CFDataRef #foreign corefoundation`
- `CFDataCreateWithBytesNoCopy :: (allocator: CFAllocatorRef, bytes: *u8, length: CFIndex, bytesDeallocator: CFAllocatorRef) -> CFDataRef #foreign corefoundation`
- `CFDataCreateCopy :: (allocator: CFAllocatorRef, theData: CFDataRef) -> CFDataRef #foreign corefoundation`
- `CFDataCreateMutable :: (allocator: CFAllocatorRef, capacity: CFIndex) -> CFMutableDataRef #foreign corefoundation`
- `CFDataCreateMutableCopy :: (allocator: CFAllocatorRef, capacity: CFIndex, theData: CFDataRef) -> CFMutableDataRef #foreign corefoundation`
- `CFDataGetLength :: (theData: CFDataRef) -> CFIndex #foreign corefoundation`
- `CFDataGetBytePtr :: (theData: CFDataRef) -> *u8 #foreign corefoundation`
- `CFDataGetMutableBytePtr :: (theData: CFMutableDataRef) -> *u8 #foreign corefoundation`
- `CFDataGetBytes :: (theData: CFDataRef, range: CFRange, buffer: *u8) -> void #foreign corefoundation`
- `CFDataSetLength :: (theData: CFMutableDataRef, length: CFIndex) -> void #foreign corefoundation`
- `CFDataIncreaseLength :: (theData: CFMutableDataRef, extraLength: CFIndex) -> void #foreign corefoundation`
- `CFDataAppendBytes :: (theData: CFMutableDataRef, bytes: *u8, length: CFIndex) -> void #foreign corefoundation`
- `CFDataReplaceBytes :: (theData: CFMutableDataRef, range: CFRange, newBytes: *u8, newLength: CFIndex) -> void #foreign corefoundation`
- `CFDataDeleteBytes :: (theData: CFMutableDataRef, range: CFRange) -> void #foreign corefoundation`
- `CFDataSearchFlags :: enum_flags CFOptionFlags`
- `CFDataFind :: (theData: CFDataRef, dataToFind: CFDataRef, searchRange: CFRange, compareOptions: CFDataSearchFlags) -> CFRange #foreign corefoundation`
- `__CFCharacterSet :: struct`
- `CFCharacterSetRef :: *__CFCharacterSet`
- `CFMutableCharacterSetRef :: *__CFCharacterSet`
- `CFCharacterSetPredefinedSet :: enum CFIndex`
- `CFCharacterSetGetTypeID :: () -> CFTypeID #foreign corefoundation`
- `CFCharacterSetGetPredefined :: (theSetIdentifier: CFCharacterSetPredefinedSet) -> CFCharacterSetRef #foreign corefoundation`
- `CFCharacterSetCreateWithCharactersInRange :: (alloc: CFAllocatorRef, theRange: CFRange) -> CFCharacterSetRef #foreign corefoundation`
- `CFCharacterSetCreateWithCharactersInString :: (alloc: CFAllocatorRef, theString: CFStringRef) -> CFCharacterSetRef #foreign corefoundation`
- `CFCharacterSetCreateWithBitmapRepresentation :: (alloc: CFAllocatorRef, theData: CFDataRef) -> CFCharacterSetRef #foreign corefoundation`
- `CFCharacterSetCreateInvertedSet :: (alloc: CFAllocatorRef, theSet: CFCharacterSetRef) -> CFCharacterSetRef #foreign corefoundation`
- `CFCharacterSetIsSupersetOfSet :: (theSet: CFCharacterSetRef, theOtherset: CFCharacterSetRef) -> Boolean #foreign corefoundation`
- `CFCharacterSetHasMemberInPlane :: (theSet: CFCharacterSetRef, thePlane: CFIndex) -> Boolean #foreign corefoundation`
- `CFCharacterSetCreateMutable :: (alloc: CFAllocatorRef) -> CFMutableCharacterSetRef #foreign corefoundation`
- `CFCharacterSetCreateCopy :: (alloc: CFAllocatorRef, theSet: CFCharacterSetRef) -> CFCharacterSetRef #foreign corefoundation`
- `CFCharacterSetCreateMutableCopy :: (alloc: CFAllocatorRef, theSet: CFCharacterSetRef) -> CFMutableCharacterSetRef #foreign corefoundation`
- `CFCharacterSetIsCharacterMember :: (theSet: CFCharacterSetRef, theChar: u16) -> Boolean #foreign corefoundation`
- `CFCharacterSetIsLongCharacterMember :: (theSet: CFCharacterSetRef, theChar: UTF32Char) -> Boolean #foreign corefoundation`
- `CFCharacterSetCreateBitmapRepresentation :: (alloc: CFAllocatorRef, theSet: CFCharacterSetRef) -> CFDataRef #foreign corefoundation`
- `CFCharacterSetAddCharactersInRange :: (theSet: CFMutableCharacterSetRef, theRange: CFRange) -> void #foreign corefoundation`
- `CFCharacterSetRemoveCharactersInRange :: (theSet: CFMutableCharacterSetRef, theRange: CFRange) -> void #foreign corefoundation`
- `CFCharacterSetAddCharactersInString :: (theSet: CFMutableCharacterSetRef, theString: CFStringRef) -> void #foreign corefoundation`
- `CFCharacterSetRemoveCharactersInString :: (theSet: CFMutableCharacterSetRef, theString: CFStringRef) -> void #foreign corefoundation`
- `CFCharacterSetUnion :: (theSet: CFMutableCharacterSetRef, theOtherSet: CFCharacterSetRef) -> void #foreign corefoundation`
- `CFCharacterSetIntersect :: (theSet: CFMutableCharacterSetRef, theOtherSet: CFCharacterSetRef) -> void #foreign corefoundation`
- `CFCharacterSetInvert :: (theSet: CFMutableCharacterSetRef) -> void #foreign corefoundation`
- `CFStringBuiltInEncodings :: CFStringEncoding`
- `CFStringEncoding :: enum u32`
- `CFStringGetTypeID :: () -> CFTypeID #foreign corefoundation`
- `CFStringCreateWithPascalString :: (alloc: CFAllocatorRef, pStr: ConstStr255Param, encoding: CFStringBuiltInEncodings) -> CFStringRef #foreign corefoundation`
- `CFStringCreateWithCString :: (alloc: CFAllocatorRef, cStr: *u8, encoding: CFStringBuiltInEncodings) -> CFStringRef #foreign corefoundation`
- `CFStringCreateWithBytes :: (alloc: CFAllocatorRef, bytes: *u8, numBytes: CFIndex, encoding: CFStringBuiltInEncodings, isExternalRepresentation: Boolean) -> CFStringRef #foreign corefoundation`
- `CFStringCreateWithCharacters :: (alloc: CFAllocatorRef, chars: *u16, numChars: CFIndex) -> CFStringRef #foreign corefoundation`
- `CFStringCreateWithPascalStringNoCopy :: (alloc: CFAllocatorRef, pStr: ConstStr255Param, encoding: CFStringBuiltInEncodings, contentsDeallocator: CFAllocatorRef) -> CFStringRef #foreign corefoundation`
- `CFStringCreateWithCStringNoCopy :: (alloc: CFAllocatorRef, cStr: *u8, encoding: CFStringBuiltInEncodings, contentsDeallocator: CFAllocatorRef) -> CFStringRef #foreign corefoundation`
- `CFStringCreateWithBytesNoCopy :: (alloc: CFAllocatorRef, bytes: *u8, numBytes: CFIndex, encoding: CFStringBuiltInEncodings, isExternalRepresentation: Boolean, contentsDeallocator: CFAllocatorRef) -> CFStringRef #foreign corefoundation`
- `CFStringCreateWithCharactersNoCopy :: (alloc: CFAllocatorRef, chars: *u16, numChars: CFIndex, contentsDeallocator: CFAllocatorRef) -> CFStringRef #foreign corefoundation`
- `CFStringCreateWithSubstring :: (alloc: CFAllocatorRef, str: CFStringRef, range: CFRange) -> CFStringRef #foreign corefoundation`
- `CFStringCreateCopy :: (alloc: CFAllocatorRef, theString: CFStringRef) -> CFStringRef #foreign corefoundation`
- `CFStringCreateWithFormat :: (alloc: CFAllocatorRef, formatOptions: CFDictionaryRef, format: CFStringRef, __args: ..Any) -> CFStringRef #foreign corefoundation`
- `CFStringCreateMutable :: (alloc: CFAllocatorRef, maxLength: CFIndex) -> CFMutableStringRef #foreign corefoundation`
- `CFStringCreateMutableCopy :: (alloc: CFAllocatorRef, maxLength: CFIndex, theString: CFStringRef) -> CFMutableStringRef #foreign corefoundation`
- `CFStringCreateMutableWithExternalCharactersNoCopy :: (alloc: CFAllocatorRef, chars: *u16, numChars: CFIndex, capacity: CFIndex, externalCharactersAllocator: CFAllocatorRef) -> CFMutableStringRef #foreign corefoundation`
- `CFStringGetLength :: (theString: CFStringRef) -> CFIndex #foreign corefoundation`
- `CFStringGetCharacterAtIndex :: (theString: CFStringRef, idx: CFIndex) -> u16 #foreign corefoundation`
- `CFStringGetCharacters :: (theString: CFStringRef, range: CFRange, buffer: *u16) -> void #foreign corefoundation`
- `CFStringGetPascalString :: (theString: CFStringRef, buffer: StringPtr, bufferSize: CFIndex, encoding: CFStringBuiltInEncodings) -> Boolean #foreign corefoundation`
- `CFStringGetCString :: (theString: CFStringRef, buffer: *u8, bufferSize: CFIndex, encoding: CFStringBuiltInEncodings) -> Boolean #foreign corefoundation`
- `CFStringGetPascalStringPtr :: (theString: CFStringRef, encoding: CFStringBuiltInEncodings) -> ConstStringPtr #foreign corefoundation`
- `CFStringGetCStringPtr :: (theString: CFStringRef, encoding: CFStringBuiltInEncodings) -> *u8 #foreign corefoundation`
- `CFStringGetCharactersPtr :: (theString: CFStringRef) -> *u16 #foreign corefoundation`
- `CFStringGetBytes :: (theString: CFStringRef, range: CFRange, encoding: CFStringBuiltInEncodings, lossByte: u8, isExternalRepresentation: Boolean, buffer: *u8, maxBufLen: CFIndex, usedBufLen: *CFIndex) -> CFIndex #foreign corefoundation`
- `CFStringCreateFromExternalRepresentation :: (alloc: CFAllocatorRef, data: CFDataRef, encoding: CFStringBuiltInEncodings) -> CFStringRef #foreign corefoundation`
- `CFStringCreateExternalRepresentation :: (alloc: CFAllocatorRef, theString: CFStringRef, encoding: CFStringBuiltInEncodings, lossByte: u8) -> CFDataRef #foreign corefoundation`
- `CFStringGetSmallestEncoding :: (theString: CFStringRef) -> CFStringBuiltInEncodings #foreign corefoundation`
- `CFStringGetFastestEncoding :: (theString: CFStringRef) -> CFStringBuiltInEncodings #foreign corefoundation`
- `CFStringGetSystemEncoding :: () -> CFStringBuiltInEncodings #foreign corefoundation`
- `CFStringGetMaximumSizeForEncoding :: (length: CFIndex, encoding: CFStringBuiltInEncodings) -> CFIndex #foreign corefoundation`
- `CFStringGetFileSystemRepresentation :: (_string: CFStringRef, buffer: *u8, maxBufLen: CFIndex) -> Boolean #foreign corefoundation`
- `CFStringGetMaximumSizeOfFileSystemRepresentation :: (_string: CFStringRef) -> CFIndex #foreign corefoundation`
- `CFStringCreateWithFileSystemRepresentation :: (alloc: CFAllocatorRef, buffer: *u8) -> CFStringRef #foreign corefoundation`
- `CFStringCompareFlags :: enum_flags CFOptionFlags`
- `CFStringCompareWithOptionsAndLocale :: (theString1: CFStringRef, theString2: CFStringRef, rangeToCompare: CFRange, compareOptions: CFStringCompareFlags, locale: CFLocaleRef) -> CFComparisonResult #foreign corefoundation`
- `CFStringCompareWithOptions :: (theString1: CFStringRef, theString2: CFStringRef, rangeToCompare: CFRange, compareOptions: CFStringCompareFlags) -> CFComparisonResult #foreign corefoundation`
- `CFStringCompare :: (theString1: CFStringRef, theString2: CFStringRef, compareOptions: CFStringCompareFlags) -> CFComparisonResult #foreign corefoundation`
- `CFStringFindWithOptionsAndLocale :: (theString: CFStringRef, stringToFind: CFStringRef, rangeToSearch: CFRange, searchOptions: CFStringCompareFlags, locale: CFLocaleRef, result: *CFRange) -> Boolean #foreign corefoundation`
- `CFStringFindWithOptions :: (theString: CFStringRef, stringToFind: CFStringRef, rangeToSearch: CFRange, searchOptions: CFStringCompareFlags, result: *CFRange) -> Boolean #foreign corefoundation`
- `CFStringCreateArrayWithFindResults :: (alloc: CFAllocatorRef, theString: CFStringRef, stringToFind: CFStringRef, rangeToSearch: CFRange, compareOptions: CFStringCompareFlags) -> CFArrayRef #foreign corefoundation`
- `CFStringFind :: (theString: CFStringRef, stringToFind: CFStringRef, compareOptions: CFStringCompareFlags) -> CFRange #foreign corefoundation`
- `CFStringHasPrefix :: (theString: CFStringRef, prefix: CFStringRef) -> Boolean #foreign corefoundation`
- `CFStringHasSuffix :: (theString: CFStringRef, suffix: CFStringRef) -> Boolean #foreign corefoundation`
- `CFStringGetRangeOfComposedCharactersAtIndex :: (theString: CFStringRef, theIndex: CFIndex) -> CFRange #foreign corefoundation`
- `CFStringFindCharacterFromSet :: (theString: CFStringRef, theSet: CFCharacterSetRef, rangeToSearch: CFRange, searchOptions: CFStringCompareFlags, result: *CFRange) -> Boolean #foreign corefoundation`
- `CFStringGetLineBounds :: (theString: CFStringRef, range: CFRange, lineBeginIndex: *CFIndex, lineEndIndex: *CFIndex, contentsEndIndex: *CFIndex) -> void #foreign corefoundation`
- `CFStringGetParagraphBounds :: (_string: CFStringRef, range: CFRange, parBeginIndex: *CFIndex, parEndIndex: *CFIndex, contentsEndIndex: *CFIndex) -> void #foreign corefoundation`
- `CFStringGetHyphenationLocationBeforeIndex :: (_string: CFStringRef, location: CFIndex, limitRange: CFRange, options: CFOptionFlags, locale: CFLocaleRef, character: *UTF32Char) -> CFIndex #foreign corefoundation`
- `CFStringIsHyphenationAvailableForLocale :: (locale: CFLocaleRef) -> Boolean #foreign corefoundation`
- `CFStringCreateByCombiningStrings :: (alloc: CFAllocatorRef, theArray: CFArrayRef, separatorString: CFStringRef) -> CFStringRef #foreign corefoundation`
- `CFStringCreateArrayBySeparatingStrings :: (alloc: CFAllocatorRef, theString: CFStringRef, separatorString: CFStringRef) -> CFArrayRef #foreign corefoundation`
- `CFStringGetIntValue :: (str: CFStringRef) -> s32 #foreign corefoundation`
- `CFStringGetDoubleValue :: (str: CFStringRef) -> float64 #foreign corefoundation`
- `CFStringAppend :: (theString: CFMutableStringRef, appendedString: CFStringRef) -> void #foreign corefoundation`
- `CFStringAppendCharacters :: (theString: CFMutableStringRef, chars: *u16, numChars: CFIndex) -> void #foreign corefoundation`
- `CFStringAppendPascalString :: (theString: CFMutableStringRef, pStr: ConstStr255Param, encoding: CFStringBuiltInEncodings) -> void #foreign corefoundation`
- `CFStringAppendCString :: (theString: CFMutableStringRef, cStr: *u8, encoding: CFStringBuiltInEncodings) -> void #foreign corefoundation`
- `CFStringAppendFormat :: (theString: CFMutableStringRef, formatOptions: CFDictionaryRef, format: CFStringRef, __args: ..Any) -> void #foreign corefoundation`
- `CFStringInsert :: (str: CFMutableStringRef, idx: CFIndex, insertedStr: CFStringRef) -> void #foreign corefoundation`
- `CFStringDelete :: (theString: CFMutableStringRef, range: CFRange) -> void #foreign corefoundation`
- `CFStringReplace :: (theString: CFMutableStringRef, range: CFRange, replacement: CFStringRef) -> void #foreign corefoundation`
- `CFStringReplaceAll :: (theString: CFMutableStringRef, replacement: CFStringRef) -> void #foreign corefoundation`
- `CFStringFindAndReplace :: (theString: CFMutableStringRef, stringToFind: CFStringRef, replacementString: CFStringRef, rangeToSearch: CFRange, compareOptions: CFStringCompareFlags) -> CFIndex #foreign corefoundation`
- `CFStringSetExternalCharactersNoCopy :: (theString: CFMutableStringRef, chars: *u16, length: CFIndex, capacity: CFIndex) -> void #foreign corefoundation`
- `CFStringPad :: (theString: CFMutableStringRef, padString: CFStringRef, length: CFIndex, indexIntoPad: CFIndex) -> void #foreign corefoundation`
- `CFStringTrim :: (theString: CFMutableStringRef, trimString: CFStringRef) -> void #foreign corefoundation`
- `CFStringTrimWhitespace :: (theString: CFMutableStringRef) -> void #foreign corefoundation`
- `CFStringLowercase :: (theString: CFMutableStringRef, locale: CFLocaleRef) -> void #foreign corefoundation`
- `CFStringUppercase :: (theString: CFMutableStringRef, locale: CFLocaleRef) -> void #foreign corefoundation`
- `CFStringCapitalize :: (theString: CFMutableStringRef, locale: CFLocaleRef) -> void #foreign corefoundation`
- `CFStringNormalizationForm :: enum CFIndex`
- `CFStringNormalize :: (theString: CFMutableStringRef, theForm: CFStringNormalizationForm) -> void #foreign corefoundation`
- `CFStringFold :: (theString: CFMutableStringRef, theFlags: CFStringCompareFlags, theLocale: CFLocaleRef) -> void #foreign corefoundation`
- `CFStringTransform :: (_string: CFMutableStringRef, range: *CFRange, transform: CFStringRef, reverse: Boolean) -> Boolean #foreign corefoundation`
- `CFStringIsEncodingAvailable :: (encoding: CFStringBuiltInEncodings) -> Boolean #foreign corefoundation`
- `CFStringGetListOfAvailableEncodings :: () -> *CFStringBuiltInEncodings #foreign corefoundation`
- `CFStringGetNameOfEncoding :: (encoding: CFStringBuiltInEncodings) -> CFStringRef #foreign corefoundation`
- `CFStringConvertEncodingToNSStringEncoding :: (encoding: CFStringBuiltInEncodings) -> u64 #foreign corefoundation`
- `CFStringConvertNSStringEncodingToEncoding :: (encoding: u64) -> CFStringBuiltInEncodings #foreign corefoundation`
- `CFStringConvertEncodingToWindowsCodepage :: (encoding: CFStringBuiltInEncodings) -> u32 #foreign corefoundation`
- `CFStringConvertWindowsCodepageToEncoding :: (codepage: u32) -> CFStringBuiltInEncodings #foreign corefoundation`
- `CFStringConvertIANACharSetNameToEncoding :: (theString: CFStringRef) -> CFStringBuiltInEncodings #foreign corefoundation`
- `CFStringConvertEncodingToIANACharSetName :: (encoding: CFStringBuiltInEncodings) -> CFStringRef #foreign corefoundation`
- `CFStringGetMostCompatibleMacStringEncoding :: (encoding: CFStringBuiltInEncodings) -> CFStringBuiltInEncodings #foreign corefoundation`
- `CFStringInlineBuffer :: struct`
- `CFShow :: (obj: CFPropertyListRef) -> void #foreign corefoundation`
- `CFShowStr :: (str: CFStringRef) -> void #foreign corefoundation`
- `__CFStringMakeConstantString :: (cStr: *u8) -> CFStringRef #foreign corefoundation`
- `CFTimeZoneGetTypeID :: () -> CFTypeID #foreign corefoundation`
- `CFTimeZoneCopySystem :: () -> CFTimeZoneRef #foreign corefoundation`
- `CFTimeZoneResetSystem :: () -> void #foreign corefoundation`
- `CFTimeZoneCopyDefault :: () -> CFTimeZoneRef #foreign corefoundation`
- `CFTimeZoneSetDefault :: (tz: CFTimeZoneRef) -> void #foreign corefoundation`
- `CFTimeZoneCopyKnownNames :: () -> CFArrayRef #foreign corefoundation`
- `CFTimeZoneCopyAbbreviationDictionary :: () -> CFDictionaryRef #foreign corefoundation`
- `CFTimeZoneSetAbbreviationDictionary :: (dict: CFDictionaryRef) -> void #foreign corefoundation`
- `CFTimeZoneCreate :: (allocator: CFAllocatorRef, name: CFStringRef, data: CFDataRef) -> CFTimeZoneRef #foreign corefoundation`
- `CFTimeZoneCreateWithTimeIntervalFromGMT :: (allocator: CFAllocatorRef, ti: CFTimeInterval) -> CFTimeZoneRef #foreign corefoundation`
- `CFTimeZoneCreateWithName :: (allocator: CFAllocatorRef, name: CFStringRef, tryAbbrev: Boolean) -> CFTimeZoneRef #foreign corefoundation`
- `CFTimeZoneGetName :: (tz: CFTimeZoneRef) -> CFStringRef #foreign corefoundation`
- `CFTimeZoneGetData :: (tz: CFTimeZoneRef) -> CFDataRef #foreign corefoundation`
- `CFTimeZoneGetSecondsFromGMT :: (tz: CFTimeZoneRef, at: CFAbsoluteTime) -> CFTimeInterval #foreign corefoundation`
- `CFTimeZoneCopyAbbreviation :: (tz: CFTimeZoneRef, at: CFAbsoluteTime) -> CFStringRef #foreign corefoundation`
- `CFTimeZoneIsDaylightSavingTime :: (tz: CFTimeZoneRef, at: CFAbsoluteTime) -> Boolean #foreign corefoundation`
- `CFTimeZoneGetDaylightSavingTimeOffset :: (tz: CFTimeZoneRef, at: CFAbsoluteTime) -> CFTimeInterval #foreign corefoundation`
- `CFTimeZoneGetNextDaylightSavingTimeTransition :: (tz: CFTimeZoneRef, at: CFAbsoluteTime) -> CFAbsoluteTime #foreign corefoundation`
- `CFTimeZoneNameStyle :: enum CFIndex`
- `CFTimeZoneCopyLocalizedName :: (tz: CFTimeZoneRef, style: CFTimeZoneNameStyle, locale: CFLocaleRef) -> CFStringRef #foreign corefoundation`
- `__CFCalendar :: struct`
- `CFCalendarRef :: *__CFCalendar`
- `CFCalendarGetTypeID :: () -> CFTypeID #foreign corefoundation`
- `CFCalendarCopyCurrent :: () -> CFCalendarRef #foreign corefoundation`
- `CFCalendarCreateWithIdentifier :: (allocator: CFAllocatorRef, identifier: CFCalendarIdentifier) -> CFCalendarRef #foreign corefoundation`
- `CFCalendarGetIdentifier :: (calendar: CFCalendarRef) -> CFCalendarIdentifier #foreign corefoundation`
- `CFCalendarCopyLocale :: (calendar: CFCalendarRef) -> CFLocaleRef #foreign corefoundation`
- `CFCalendarSetLocale :: (calendar: CFCalendarRef, locale: CFLocaleRef) -> void #foreign corefoundation`
- `CFCalendarCopyTimeZone :: (calendar: CFCalendarRef) -> CFTimeZoneRef #foreign corefoundation`
- `CFCalendarSetTimeZone :: (calendar: CFCalendarRef, tz: CFTimeZoneRef) -> void #foreign corefoundation`
- `CFCalendarGetFirstWeekday :: (calendar: CFCalendarRef) -> CFIndex #foreign corefoundation`
- `CFCalendarSetFirstWeekday :: (calendar: CFCalendarRef, wkdy: CFIndex) -> void #foreign corefoundation`
- `CFCalendarGetMinimumDaysInFirstWeek :: (calendar: CFCalendarRef) -> CFIndex #foreign corefoundation`
- `CFCalendarSetMinimumDaysInFirstWeek :: (calendar: CFCalendarRef, mwd: CFIndex) -> void #foreign corefoundation`
- `CFCalendarUnit :: enum CFOptionFlags`
- `CFCalendarGetMinimumRangeOfUnit :: (calendar: CFCalendarRef, unit: CFCalendarUnit) -> CFRange #foreign corefoundation`
- `CFCalendarGetMaximumRangeOfUnit :: (calendar: CFCalendarRef, unit: CFCalendarUnit) -> CFRange #foreign corefoundation`
- `CFCalendarGetRangeOfUnit :: (calendar: CFCalendarRef, smallerUnit: CFCalendarUnit, biggerUnit: CFCalendarUnit, at: CFAbsoluteTime) -> CFRange #foreign corefoundation`
- `CFCalendarGetOrdinalityOfUnit :: (calendar: CFCalendarRef, smallerUnit: CFCalendarUnit, biggerUnit: CFCalendarUnit, at: CFAbsoluteTime) -> CFIndex #foreign corefoundation`
- `CFCalendarGetTimeRangeOfUnit :: (calendar: CFCalendarRef, unit: CFCalendarUnit, at: CFAbsoluteTime, startp: *CFAbsoluteTime, tip: *CFTimeInterval) -> Boolean #foreign corefoundation`
- `CFCalendarComposeAbsoluteTime :: (calendar: CFCalendarRef, at: *CFAbsoluteTime, componentDesc: *u8, __args: ..Any) -> Boolean #foreign corefoundation`
- `CFCalendarDecomposeAbsoluteTime :: (calendar: CFCalendarRef, at: CFAbsoluteTime, componentDesc: *u8, __args: ..Any) -> Boolean #foreign corefoundation`
- `anon_enum_6 :: enum u32`
- `CFCalendarAddComponents :: (calendar: CFCalendarRef, at: *CFAbsoluteTime, options: CFOptionFlags, componentDesc: *u8, __args: ..Any) -> Boolean #foreign corefoundation`
- `CFCalendarGetComponentDifference :: (calendar: CFCalendarRef, startingAT: CFAbsoluteTime, resultAT: CFAbsoluteTime, options: CFOptionFlags, componentDesc: *u8, __args: ..Any) -> Boolean #foreign corefoundation`
- `CFDateFormatterKey :: CFStringRef`
- `__CFDateFormatter :: struct`
- `CFDateFormatterRef :: *__CFDateFormatter`
- `CFDateFormatterCreateDateFormatFromTemplate :: (allocator: CFAllocatorRef, tmplate: CFStringRef, options: CFOptionFlags, locale: CFLocaleRef) -> CFStringRef #foreign corefoundation`
- `CFDateFormatterGetTypeID :: () -> CFTypeID #foreign corefoundation`
- `CFDateFormatterStyle :: CFIndex`
- `kCFDateFormatter :: enum u32`
- `CFISO8601DateFormatOptions :: CFOptionFlags`
- `kCFISO8601DateFormatWith :: enum u32`
- `CFDateFormatterCreateISO8601Formatter :: (allocator: CFAllocatorRef, formatOptions: CFISO8601DateFormatOptions) -> CFDateFormatterRef #foreign corefoundation`
- `CFDateFormatterCreate :: (allocator: CFAllocatorRef, locale: CFLocaleRef, dateStyle: CFDateFormatterStyle, timeStyle: CFDateFormatterStyle) -> CFDateFormatterRef #foreign corefoundation`
- `CFDateFormatterGetLocale :: (formatter: CFDateFormatterRef) -> CFLocaleRef #foreign corefoundation`
- `CFDateFormatterGetDateStyle :: (formatter: CFDateFormatterRef) -> CFDateFormatterStyle #foreign corefoundation`
- `CFDateFormatterGetTimeStyle :: (formatter: CFDateFormatterRef) -> CFDateFormatterStyle #foreign corefoundation`
- `CFDateFormatterGetFormat :: (formatter: CFDateFormatterRef) -> CFStringRef #foreign corefoundation`
- `CFDateFormatterSetFormat :: (formatter: CFDateFormatterRef, formatString: CFStringRef) -> void #foreign corefoundation`
- `CFDateFormatterCreateStringWithDate :: (allocator: CFAllocatorRef, formatter: CFDateFormatterRef, date: CFDateRef) -> CFStringRef #foreign corefoundation`
- `CFDateFormatterCreateStringWithAbsoluteTime :: (allocator: CFAllocatorRef, formatter: CFDateFormatterRef, at: CFAbsoluteTime) -> CFStringRef #foreign corefoundation`
- `CFDateFormatterCreateDateFromString :: (allocator: CFAllocatorRef, formatter: CFDateFormatterRef, _string: CFStringRef, rangep: *CFRange) -> CFDateRef #foreign corefoundation`
- `CFDateFormatterGetAbsoluteTimeFromString :: (formatter: CFDateFormatterRef, _string: CFStringRef, rangep: *CFRange, atp: *CFAbsoluteTime) -> Boolean #foreign corefoundation`
- `CFDateFormatterSetProperty :: (formatter: CFDateFormatterRef, key: CFStringRef, value: CFPropertyListRef) -> void #foreign corefoundation`
- `CFDateFormatterCopyProperty :: (formatter: CFDateFormatterRef, key: CFDateFormatterKey) -> CFPropertyListRef #foreign corefoundation`
- `CFErrorDomain :: CFStringRef`
- `__CFError :: struct`
- `CFErrorRef :: *__CFError`
- `CFErrorGetTypeID :: () -> CFTypeID #foreign corefoundation`
- `CFErrorCreate :: (allocator: CFAllocatorRef, domain: CFErrorDomain, code: CFIndex, userInfo: CFDictionaryRef) -> CFErrorRef #foreign corefoundation`
- `CFErrorCreateWithUserInfoKeysAndValues :: (allocator: CFAllocatorRef, domain: CFErrorDomain, code: CFIndex, userInfoKeys: **void, userInfoValues: **void, numUserInfoValues: CFIndex) -> CFErrorRef #foreign corefoundation`
- `CFErrorGetDomain :: (err: CFErrorRef) -> CFErrorDomain #foreign corefoundation`
- `CFErrorGetCode :: (err: CFErrorRef) -> CFIndex #foreign corefoundation`
- `CFErrorCopyUserInfo :: (err: CFErrorRef) -> CFDictionaryRef #foreign corefoundation`
- `CFErrorCopyDescription :: (err: CFErrorRef) -> CFStringRef #foreign corefoundation`
- `CFErrorCopyFailureReason :: (err: CFErrorRef) -> CFStringRef #foreign corefoundation`
- `CFErrorCopyRecoverySuggestion :: (err: CFErrorRef) -> CFStringRef #foreign corefoundation`
- `__CFBoolean :: struct`
- `CFBooleanRef :: *__CFBoolean`
- `CFBooleanGetTypeID :: () -> CFTypeID #foreign corefoundation`
- `CFBooleanGetValue :: (boolean: CFBooleanRef) -> Boolean #foreign corefoundation`
- `CFNumberType :: CFIndex`
- `kCFNumber :: enum u32`
- `__CFNumber :: struct`
- `CFNumberRef :: *__CFNumber`
- `CFNumberGetTypeID :: () -> CFTypeID #foreign corefoundation`
- `CFNumberCreate :: (allocator: CFAllocatorRef, theType: CFNumberType, valuePtr: *void) -> CFNumberRef #foreign corefoundation`
- `CFNumberGetType :: (number: CFNumberRef) -> CFNumberType #foreign corefoundation`
- `CFNumberGetByteSize :: (number: CFNumberRef) -> CFIndex #foreign corefoundation`
- `CFNumberIsFloatType :: (number: CFNumberRef) -> Boolean #foreign corefoundation`
- `CFNumberGetValue :: (number: CFNumberRef, theType: CFNumberType, valuePtr: *void) -> Boolean #foreign corefoundation`
- `CFNumberCompare :: (number: CFNumberRef, otherNumber: CFNumberRef, _context: *void) -> CFComparisonResult #foreign corefoundation`
- `CFNumberFormatterKey :: CFStringRef`
- `__CFNumberFormatter :: struct`
- `CFNumberFormatterRef :: *__CFNumberFormatter`
- `CFNumberFormatterGetTypeID :: () -> CFTypeID #foreign corefoundation`
- `CFNumberFormatterStyle :: CFIndex`
- `kCFNumberFormatter :: enum u32`
- `CFNumberFormatterCreate :: (allocator: CFAllocatorRef, locale: CFLocaleRef, style: CFNumberFormatterStyle) -> CFNumberFormatterRef #foreign corefoundation`
- `CFNumberFormatterGetLocale :: (formatter: CFNumberFormatterRef) -> CFLocaleRef #foreign corefoundation`
- `CFNumberFormatterGetStyle :: (formatter: CFNumberFormatterRef) -> CFNumberFormatterStyle #foreign corefoundation`
- `CFNumberFormatterGetFormat :: (formatter: CFNumberFormatterRef) -> CFStringRef #foreign corefoundation`
- `CFNumberFormatterSetFormat :: (formatter: CFNumberFormatterRef, formatString: CFStringRef) -> void #foreign corefoundation`
- `CFNumberFormatterCreateStringWithNumber :: (allocator: CFAllocatorRef, formatter: CFNumberFormatterRef, number: CFNumberRef) -> CFStringRef #foreign corefoundation`
- `CFNumberFormatterCreateStringWithValue :: (allocator: CFAllocatorRef, formatter: CFNumberFormatterRef, numberType: CFNumberType, valuePtr: *void) -> CFStringRef #foreign corefoundation`
- `CFNumberFormatterOptionFlags :: CFOptionFlags`
- `anon_enum_7 :: enum u32`
- `CFNumberFormatterCreateNumberFromString :: (allocator: CFAllocatorRef, formatter: CFNumberFormatterRef, _string: CFStringRef, rangep: *CFRange, options: CFOptionFlags) -> CFNumberRef #foreign corefoundation`
- `CFNumberFormatterGetValueFromString :: (formatter: CFNumberFormatterRef, _string: CFStringRef, rangep: *CFRange, numberType: CFNumberType, valuePtr: *void) -> Boolean #foreign corefoundation`
- `CFNumberFormatterSetProperty :: (formatter: CFNumberFormatterRef, key: CFNumberFormatterKey, value: CFPropertyListRef) -> void #foreign corefoundation`
- `CFNumberFormatterCopyProperty :: (formatter: CFNumberFormatterRef, key: CFNumberFormatterKey) -> CFPropertyListRef #foreign corefoundation`
- `CFNumberFormatterRoundingMode :: CFIndex`
- `kCFNumberFormatterRound :: enum u32`
- `CFNumberFormatterPadPosition :: CFIndex`
- `kCFNumberFormatterPad :: enum u32`
- `CFNumberFormatterGetDecimalInfoForCurrencyCode :: (currencyCode: CFStringRef, defaultFractionDigits: *s32, roundingIncrement: *float64) -> Boolean #foreign corefoundation`
- `CFPreferencesCopyAppValue :: (key: CFStringRef, applicationID: CFStringRef) -> CFPropertyListRef #foreign corefoundation`
- `CFPreferencesGetAppBooleanValue :: (key: CFStringRef, applicationID: CFStringRef, keyExistsAndHasValidFormat: *Boolean) -> Boolean #foreign corefoundation`
- `CFPreferencesGetAppIntegerValue :: (key: CFStringRef, applicationID: CFStringRef, keyExistsAndHasValidFormat: *Boolean) -> CFIndex #foreign corefoundation`
- `CFPreferencesSetAppValue :: (key: CFStringRef, value: CFPropertyListRef, applicationID: CFStringRef) -> void #foreign corefoundation`
- `CFPreferencesAddSuitePreferencesToApp :: (applicationID: CFStringRef, suiteID: CFStringRef) -> void #foreign corefoundation`
- `CFPreferencesRemoveSuitePreferencesFromApp :: (applicationID: CFStringRef, suiteID: CFStringRef) -> void #foreign corefoundation`
- `CFPreferencesAppSynchronize :: (applicationID: CFStringRef) -> Boolean #foreign corefoundation`
- `CFPreferencesCopyValue :: (key: CFStringRef, applicationID: CFStringRef, userName: CFStringRef, hostName: CFStringRef) -> CFPropertyListRef #foreign corefoundation`
- `CFPreferencesCopyMultiple :: (keysToFetch: CFArrayRef, applicationID: CFStringRef, userName: CFStringRef, hostName: CFStringRef) -> CFDictionaryRef #foreign corefoundation`
- `CFPreferencesSetValue :: (key: CFStringRef, value: CFPropertyListRef, applicationID: CFStringRef, userName: CFStringRef, hostName: CFStringRef) -> void #foreign corefoundation`
- `CFPreferencesSetMultiple :: (keysToSet: CFDictionaryRef, keysToRemove: CFArrayRef, applicationID: CFStringRef, userName: CFStringRef, hostName: CFStringRef) -> void #foreign corefoundation`
- `CFPreferencesSynchronize :: (applicationID: CFStringRef, userName: CFStringRef, hostName: CFStringRef) -> Boolean #foreign corefoundation`
- `CFPreferencesCopyApplicationList :: (userName: CFStringRef, hostName: CFStringRef) -> CFArrayRef #foreign corefoundation`
- `CFPreferencesCopyKeyList :: (applicationID: CFStringRef, userName: CFStringRef, hostName: CFStringRef) -> CFArrayRef #foreign corefoundation`
- `CFPreferencesAppValueIsForced :: (key: CFStringRef, applicationID: CFStringRef) -> Boolean #foreign corefoundation`
- `CFURLPathStyle :: CFIndex`
- `kCFURL :: enum u32`
- `__CFURL :: struct`
- `CFURLRef :: *__CFURL`
- `CFURLGetTypeID :: () -> CFTypeID #foreign corefoundation`
- `CFURLCreateWithBytes :: (allocator: CFAllocatorRef, URLBytes: *u8, length: CFIndex, encoding: CFStringBuiltInEncodings, baseURL: CFURLRef) -> CFURLRef #foreign corefoundation`
- `CFURLCreateData :: (allocator: CFAllocatorRef, url: CFURLRef, encoding: CFStringBuiltInEncodings, escapeWhitespace: Boolean) -> CFDataRef #foreign corefoundation`
- `CFURLCreateWithString :: (allocator: CFAllocatorRef, URLString: CFStringRef, baseURL: CFURLRef) -> CFURLRef #foreign corefoundation`
- `CFURLCreateAbsoluteURLWithBytes :: (alloc: CFAllocatorRef, relativeURLBytes: *u8, length: CFIndex, encoding: CFStringBuiltInEncodings, baseURL: CFURLRef, useCompatibilityMode: Boolean) -> CFURLRef #foreign corefoundation`
- `CFURLCreateWithFileSystemPath :: (allocator: CFAllocatorRef, filePath: CFStringRef, pathStyle: CFURLPathStyle, isDirectory: Boolean) -> CFURLRef #foreign corefoundation`
- `CFURLCreateFromFileSystemRepresentation :: (allocator: CFAllocatorRef, buffer: *u8, bufLen: CFIndex, isDirectory: Boolean) -> CFURLRef #foreign corefoundation`
- `CFURLCreateWithFileSystemPathRelativeToBase :: (allocator: CFAllocatorRef, filePath: CFStringRef, pathStyle: CFURLPathStyle, isDirectory: Boolean, baseURL: CFURLRef) -> CFURLRef #foreign corefoundation`
- `CFURLCreateFromFileSystemRepresentationRelativeToBase :: (allocator: CFAllocatorRef, buffer: *u8, bufLen: CFIndex, isDirectory: Boolean, baseURL: CFURLRef) -> CFURLRef #foreign corefoundation`
- `CFURLGetFileSystemRepresentation :: (url: CFURLRef, resolveAgainstBase: Boolean, buffer: *u8, maxBufLen: CFIndex) -> Boolean #foreign corefoundation`
- `CFURLCopyAbsoluteURL :: (relativeURL: CFURLRef) -> CFURLRef #foreign corefoundation`
- `CFURLGetString :: (anURL: CFURLRef) -> CFStringRef #foreign corefoundation`
- `CFURLGetBaseURL :: (anURL: CFURLRef) -> CFURLRef #foreign corefoundation`
- `CFURLCanBeDecomposed :: (anURL: CFURLRef) -> Boolean #foreign corefoundation`
- `CFURLCopyScheme :: (anURL: CFURLRef) -> CFStringRef #foreign corefoundation`
- `CFURLCopyNetLocation :: (anURL: CFURLRef) -> CFStringRef #foreign corefoundation`
- `CFURLCopyPath :: (anURL: CFURLRef) -> CFStringRef #foreign corefoundation`
- `CFURLCopyStrictPath :: (anURL: CFURLRef, isAbsolute: *Boolean) -> CFStringRef #foreign corefoundation`
- `CFURLCopyFileSystemPath :: (anURL: CFURLRef, pathStyle: CFURLPathStyle) -> CFStringRef #foreign corefoundation`
- `CFURLHasDirectoryPath :: (anURL: CFURLRef) -> Boolean #foreign corefoundation`
- `CFURLCopyResourceSpecifier :: (anURL: CFURLRef) -> CFStringRef #foreign corefoundation`
- `CFURLCopyHostName :: (anURL: CFURLRef) -> CFStringRef #foreign corefoundation`
- `CFURLGetPortNumber :: (anURL: CFURLRef) -> s32 #foreign corefoundation`
- `CFURLCopyUserName :: (anURL: CFURLRef) -> CFStringRef #foreign corefoundation`
- `CFURLCopyPassword :: (anURL: CFURLRef) -> CFStringRef #foreign corefoundation`
- `CFURLCopyParameterString :: (anURL: CFURLRef, charactersToLeaveEscaped: CFStringRef) -> CFStringRef #foreign corefoundation`
- `CFURLCopyQueryString :: (anURL: CFURLRef, charactersToLeaveEscaped: CFStringRef) -> CFStringRef #foreign corefoundation`
- `CFURLCopyFragment :: (anURL: CFURLRef, charactersToLeaveEscaped: CFStringRef) -> CFStringRef #foreign corefoundation`
- `CFURLCopyLastPathComponent :: (url: CFURLRef) -> CFStringRef #foreign corefoundation`
- `CFURLCopyPathExtension :: (url: CFURLRef) -> CFStringRef #foreign corefoundation`
- `CFURLCreateCopyAppendingPathComponent :: (allocator: CFAllocatorRef, url: CFURLRef, pathComponent: CFStringRef, isDirectory: Boolean) -> CFURLRef #foreign corefoundation`
- `CFURLCreateCopyDeletingLastPathComponent :: (allocator: CFAllocatorRef, url: CFURLRef) -> CFURLRef #foreign corefoundation`
- `CFURLCreateCopyAppendingPathExtension :: (allocator: CFAllocatorRef, url: CFURLRef, extension: CFStringRef) -> CFURLRef #foreign corefoundation`
- `CFURLCreateCopyDeletingPathExtension :: (allocator: CFAllocatorRef, url: CFURLRef) -> CFURLRef #foreign corefoundation`
- `CFURLGetBytes :: (url: CFURLRef, buffer: *u8, bufferLength: CFIndex) -> CFIndex #foreign corefoundation`
- `CFURLComponentType :: CFIndex`
- `kCFURLComponent :: enum u32`
- `CFURLGetByteRangeForComponent :: (url: CFURLRef, component: CFURLComponentType, rangeIncludingSeparators: *CFRange) -> CFRange #foreign corefoundation`
- `CFURLCreateStringByReplacingPercentEscapes :: (allocator: CFAllocatorRef, originalString: CFStringRef, charactersToLeaveEscaped: CFStringRef) -> CFStringRef #foreign corefoundation`
- `CFURLCreateStringByReplacingPercentEscapesUsingEncoding :: (allocator: CFAllocatorRef, origString: CFStringRef, charsToLeaveEscaped: CFStringRef, encoding: CFStringBuiltInEncodings) -> CFStringRef #foreign corefoundation`
- `CFURLCreateStringByAddingPercentEscapes :: (allocator: CFAllocatorRef, originalString: CFStringRef, charactersToLeaveUnescaped: CFStringRef, legalURLCharactersToBeEscaped: CFStringRef, encoding: CFStringBuiltInEncodings) -> CFStringRef #foreign corefoundation`
- `CFURLIsFileReferenceURL :: (url: CFURLRef) -> Boolean #foreign corefoundation`
- `CFURLCreateFileReferenceURL :: (allocator: CFAllocatorRef, url: CFURLRef, error: *CFErrorRef) -> CFURLRef #foreign corefoundation`
- `CFURLCreateFilePathURL :: (allocator: CFAllocatorRef, url: CFURLRef, error: *CFErrorRef) -> CFURLRef #foreign corefoundation`
- `FSRef :: struct`
- `CFURLCreateFromFSRef :: (allocator: CFAllocatorRef, fsRef: *FSRef) -> CFURLRef #foreign corefoundation`
- `CFURLGetFSRef :: (url: CFURLRef, fsRef: *FSRef) -> Boolean #foreign corefoundation`
- `CFURLCopyResourcePropertyForKey :: (url: CFURLRef, key: CFStringRef, propertyValueTypeRefPtr: *void, error: *CFErrorRef) -> Boolean #foreign corefoundation`
- `CFURLCopyResourcePropertiesForKeys :: (url: CFURLRef, keys: CFArrayRef, error: *CFErrorRef) -> CFDictionaryRef #foreign corefoundation`
- `CFURLSetResourcePropertyForKey :: (url: CFURLRef, key: CFStringRef, propertyValue: CFPropertyListRef, error: *CFErrorRef) -> Boolean #foreign corefoundation`
- `CFURLSetResourcePropertiesForKeys :: (url: CFURLRef, keyedPropertyValues: CFDictionaryRef, error: *CFErrorRef) -> Boolean #foreign corefoundation`
- `CFURLClearResourcePropertyCacheForKey :: (url: CFURLRef, key: CFStringRef) -> void #foreign corefoundation`
- `CFURLClearResourcePropertyCache :: (url: CFURLRef) -> void #foreign corefoundation`
- `CFURLSetTemporaryResourcePropertyForKey :: (url: CFURLRef, key: CFStringRef, propertyValue: CFPropertyListRef) -> void #foreign corefoundation`
- `CFURLResourceIsReachable :: (url: CFURLRef, error: *CFErrorRef) -> Boolean #foreign corefoundation`
- `CFURLBookmarkCreationOptions :: CFOptionFlags`
- `kCFURLBookmarkCreation :: enum u32`
- `CFURLBookmarkResolutionOptions :: CFOptionFlags`
- `kCF :: enum u32`
- `CFURLBookmarkFileCreationOptions :: CFOptionFlags`
- `CFURLCreateBookmarkData :: (allocator: CFAllocatorRef, url: CFURLRef, options: CFURLBookmarkCreationOptions, resourcePropertiesToInclude: CFArrayRef, relativeToURL: CFURLRef, error: *CFErrorRef) -> CFDataRef #foreign corefoundation`
- `CFURLCreateByResolvingBookmarkData :: (allocator: CFAllocatorRef, bookmark: CFDataRef, options: CFURLBookmarkResolutionOptions, relativeToURL: CFURLRef, resourcePropertiesToInclude: CFArrayRef, isStale: *Boolean, error: *CFErrorRef) -> CFURLRef #foreign corefoundation`
- `CFURLCreateResourcePropertiesForKeysFromBookmarkData :: (allocator: CFAllocatorRef, resourcePropertiesToReturn: CFArrayRef, bookmark: CFDataRef) -> CFDictionaryRef #foreign corefoundation`
- `CFURLCreateResourcePropertyForKeyFromBookmarkData :: (allocator: CFAllocatorRef, resourcePropertyKey: CFStringRef, bookmark: CFDataRef) -> CFPropertyListRef #foreign corefoundation`
- `CFURLCreateBookmarkDataFromFile :: (allocator: CFAllocatorRef, fileURL: CFURLRef, errorRef: *CFErrorRef) -> CFDataRef #foreign corefoundation`
- `CFURLWriteBookmarkDataToFile :: (bookmarkRef: CFDataRef, fileURL: CFURLRef, options: CFURLBookmarkFileCreationOptions, errorRef: *CFErrorRef) -> Boolean #foreign corefoundation`
- `CFURLCreateBookmarkDataFromAliasRecord :: (allocatorRef: CFAllocatorRef, aliasRecordDataRef: CFDataRef) -> CFDataRef #foreign corefoundation`
- `CFURLStartAccessingSecurityScopedResource :: (url: CFURLRef) -> Boolean #foreign corefoundation`
- `CFURLStopAccessingSecurityScopedResource :: (url: CFURLRef) -> void #foreign corefoundation`
- `CFRunLoopMode :: CFStringRef`
- `__CFRunLoop :: struct`
- `CFRunLoopRef :: *__CFRunLoop`
- `__CFRunLoopSource :: struct`
- `CFRunLoopSourceRef :: *__CFRunLoopSource`
- `__CFRunLoopObserver :: struct`
- `CFRunLoopObserverRef :: *__CFRunLoopObserver`
- `__CFRunLoopTimer :: struct`
- `CFRunLoopTimerRef :: *__CFRunLoopTimer`
- `CFRunLoopRunResult :: enum s32`
- `CFRunLoopActivity :: enum_flags CFOptionFlags`
- `CFRunLoopGetTypeID :: () -> CFTypeID #foreign corefoundation`
- `CFRunLoopGetCurrent :: () -> CFRunLoopRef #foreign corefoundation`
- `CFRunLoopGetMain :: () -> CFRunLoopRef #foreign corefoundation`
- `CFRunLoopCopyCurrentMode :: (rl: CFRunLoopRef) -> CFRunLoopMode #foreign corefoundation`
- `CFRunLoopCopyAllModes :: (rl: CFRunLoopRef) -> CFArrayRef #foreign corefoundation`
- `CFRunLoopAddCommonMode :: (rl: CFRunLoopRef, mode: CFRunLoopMode) -> void #foreign corefoundation`
- `CFRunLoopGetNextTimerFireDate :: (rl: CFRunLoopRef, mode: CFRunLoopMode) -> CFAbsoluteTime #foreign corefoundation`
- `CFRunLoopRun :: () -> void #foreign corefoundation`
- `CFRunLoopRunInMode :: (mode: CFRunLoopMode, seconds: CFTimeInterval, returnAfterSourceHandled: Boolean) -> CFRunLoopRunResult #foreign corefoundation`
- `CFRunLoopIsWaiting :: (rl: CFRunLoopRef) -> Boolean #foreign corefoundation`
- `CFRunLoopWakeUp :: (rl: CFRunLoopRef) -> void #foreign corefoundation`
- `CFRunLoopStop :: (rl: CFRunLoopRef) -> void #foreign corefoundation`
- `CFRunLoopPerformBlock :: (rl: CFRunLoopRef, mode: CFPropertyListRef, block: Block) -> void #foreign corefoundation`
- `CFRunLoopContainsSource :: (rl: CFRunLoopRef, source: CFRunLoopSourceRef, mode: CFRunLoopMode) -> Boolean #foreign corefoundation`
- `CFRunLoopAddSource :: (rl: CFRunLoopRef, source: CFRunLoopSourceRef, mode: CFRunLoopMode) -> void #foreign corefoundation`
- `CFRunLoopRemoveSource :: (rl: CFRunLoopRef, source: CFRunLoopSourceRef, mode: CFRunLoopMode) -> void #foreign corefoundation`
- `CFRunLoopContainsObserver :: (rl: CFRunLoopRef, observer: CFRunLoopObserverRef, mode: CFRunLoopMode) -> Boolean #foreign corefoundation`
- `CFRunLoopAddObserver :: (rl: CFRunLoopRef, observer: CFRunLoopObserverRef, mode: CFRunLoopMode) -> void #foreign corefoundation`
- `CFRunLoopRemoveObserver :: (rl: CFRunLoopRef, observer: CFRunLoopObserverRef, mode: CFRunLoopMode) -> void #foreign corefoundation`
- `CFRunLoopContainsTimer :: (rl: CFRunLoopRef, timer: CFRunLoopTimerRef, mode: CFRunLoopMode) -> Boolean #foreign corefoundation`
- `CFRunLoopAddTimer :: (rl: CFRunLoopRef, timer: CFRunLoopTimerRef, mode: CFRunLoopMode) -> void #foreign corefoundation`
- `CFRunLoopRemoveTimer :: (rl: CFRunLoopRef, timer: CFRunLoopTimerRef, mode: CFRunLoopMode) -> void #foreign corefoundation`
- `CFRunLoopSourceContext :: struct`
- `CFRunLoopSourceContext1 :: struct`
- `CFRunLoopSourceGetTypeID :: () -> CFTypeID #foreign corefoundation`
- `CFRunLoopSourceCreate :: (allocator: CFAllocatorRef, order: CFIndex, _context: *CFRunLoopSourceContext) -> CFRunLoopSourceRef #foreign corefoundation`
- `CFRunLoopSourceGetOrder :: (source: CFRunLoopSourceRef) -> CFIndex #foreign corefoundation`
- `CFRunLoopSourceInvalidate :: (source: CFRunLoopSourceRef) -> void #foreign corefoundation`
- `CFRunLoopSourceIsValid :: (source: CFRunLoopSourceRef) -> Boolean #foreign corefoundation`
- `CFRunLoopSourceGetContext :: (source: CFRunLoopSourceRef, _context: *CFRunLoopSourceContext) -> void #foreign corefoundation`
- `CFRunLoopSourceSignal :: (source: CFRunLoopSourceRef) -> void #foreign corefoundation`
- `CFRunLoopObserverContext :: struct`
- `CFRunLoopObserverCallBack :: #type (observer: CFRunLoopObserverRef, activity: CFRunLoopActivity, info: *void) -> void #c_call`
- `CFRunLoopObserverGetTypeID :: () -> CFTypeID #foreign corefoundation`
- `CFRunLoopObserverCreate :: (allocator: CFAllocatorRef, activities: CFOptionFlags, repeats: Boolean, order: CFIndex, callout: CFRunLoopObserverCallBack, _context: *CFRunLoopObserverContext) -> CFRunLoopObserverRef #foreign corefoundation`
- `CFRunLoopObserverCreateWithHandler :: (allocator: CFAllocatorRef, activities: CFOptionFlags, repeats: Boolean, order: CFIndex, block: Block) -> CFRunLoopObserverRef #foreign corefoundation`
- `CFRunLoopObserverGetActivities :: (observer: CFRunLoopObserverRef) -> CFOptionFlags #foreign corefoundation`
- `CFRunLoopObserverDoesRepeat :: (observer: CFRunLoopObserverRef) -> Boolean #foreign corefoundation`
- `CFRunLoopObserverGetOrder :: (observer: CFRunLoopObserverRef) -> CFIndex #foreign corefoundation`
- `CFRunLoopObserverInvalidate :: (observer: CFRunLoopObserverRef) -> void #foreign corefoundation`
- `CFRunLoopObserverIsValid :: (observer: CFRunLoopObserverRef) -> Boolean #foreign corefoundation`
- `CFRunLoopObserverGetContext :: (observer: CFRunLoopObserverRef, _context: *CFRunLoopObserverContext) -> void #foreign corefoundation`
- `CFRunLoopTimerContext :: struct`
- `CFRunLoopTimerCallBack :: #type (timer: CFRunLoopTimerRef, info: *void) -> void #c_call`
- `CFRunLoopTimerGetTypeID :: () -> CFTypeID #foreign corefoundation`
- `CFRunLoopTimerCreate :: (allocator: CFAllocatorRef, fireDate: CFAbsoluteTime, interval: CFTimeInterval, flags: CFOptionFlags, order: CFIndex, callout: CFRunLoopTimerCallBack, _context: *CFRunLoopTimerContext) -> CFRunLoopTimerRef #foreign corefoundation`
- `CFRunLoopTimerCreateWithHandler :: (allocator: CFAllocatorRef, fireDate: CFAbsoluteTime, interval: CFTimeInterval, flags: CFOptionFlags, order: CFIndex, block: Block) -> CFRunLoopTimerRef #foreign corefoundation`
- `CFRunLoopTimerGetNextFireDate :: (timer: CFRunLoopTimerRef) -> CFAbsoluteTime #foreign corefoundation`
- `CFRunLoopTimerSetNextFireDate :: (timer: CFRunLoopTimerRef, fireDate: CFAbsoluteTime) -> void #foreign corefoundation`
- `CFRunLoopTimerGetInterval :: (timer: CFRunLoopTimerRef) -> CFTimeInterval #foreign corefoundation`
- `CFRunLoopTimerDoesRepeat :: (timer: CFRunLoopTimerRef) -> Boolean #foreign corefoundation`
- `CFRunLoopTimerGetOrder :: (timer: CFRunLoopTimerRef) -> CFIndex #foreign corefoundation`
- `CFRunLoopTimerInvalidate :: (timer: CFRunLoopTimerRef) -> void #foreign corefoundation`
- `CFRunLoopTimerIsValid :: (timer: CFRunLoopTimerRef) -> Boolean #foreign corefoundation`
- `CFRunLoopTimerGetContext :: (timer: CFRunLoopTimerRef, _context: *CFRunLoopTimerContext) -> void #foreign corefoundation`
- `CFRunLoopTimerGetTolerance :: (timer: CFRunLoopTimerRef) -> CFTimeInterval #foreign corefoundation`
- `CFRunLoopTimerSetTolerance :: (timer: CFRunLoopTimerRef, tolerance: CFTimeInterval) -> void #foreign corefoundation`
- `__CFSocket :: struct`
- `CFSocketRef :: *__CFSocket`
- `CFSocketError :: CFIndex`
- `kCFSocket :: enum s32`
- `CFSocketSignature :: struct`
- `CFSocketCallBackType :: CFOptionFlags`
- `kCFSocket_1 :: enum u32`
- `kCFSocket_2 :: enum u32`
- `CFSocketCallBack :: #type (s: CFSocketRef, type: CFSocketCallBackType, address: CFDataRef, data: *void, info: *void) -> void #c_call`
- `CFSocketContext :: struct`
- `CFSocketNativeHandle :: s32`
- `CFSocketGetTypeID :: () -> CFTypeID #foreign corefoundation`
- `CFSocketCreate :: (allocator: CFAllocatorRef, protocolFamily: s32, socketType: s32, protocol: s32, callBackTypes: CFOptionFlags, callout: CFSocketCallBack, _context: *CFSocketContext) -> CFSocketRef #foreign corefoundation`
- `CFSocketCreateWithNative :: (allocator: CFAllocatorRef, sock: CFSocketNativeHandle, callBackTypes: CFOptionFlags, callout: CFSocketCallBack, _context: *CFSocketContext) -> CFSocketRef #foreign corefoundation`
- `CFSocketCreateWithSocketSignature :: (allocator: CFAllocatorRef, signature: *CFSocketSignature, callBackTypes: CFOptionFlags, callout: CFSocketCallBack, _context: *CFSocketContext) -> CFSocketRef #foreign corefoundation`
- `CFSocketCreateConnectedToSocketSignature :: (allocator: CFAllocatorRef, signature: *CFSocketSignature, callBackTypes: CFOptionFlags, callout: CFSocketCallBack, _context: *CFSocketContext, timeout: CFTimeInterval) -> CFSocketRef #foreign corefoundation`
- `CFSocketSetAddress :: (s: CFSocketRef, address: CFDataRef) -> CFSocketError #foreign corefoundation`
- `CFSocketConnectToAddress :: (s: CFSocketRef, address: CFDataRef, timeout: CFTimeInterval) -> CFSocketError #foreign corefoundation`
- `CFSocketInvalidate :: (s: CFSocketRef) -> void #foreign corefoundation`
- `CFSocketIsValid :: (s: CFSocketRef) -> Boolean #foreign corefoundation`
- `CFSocketCopyAddress :: (s: CFSocketRef) -> CFDataRef #foreign corefoundation`
- `CFSocketCopyPeerAddress :: (s: CFSocketRef) -> CFDataRef #foreign corefoundation`
- `CFSocketGetContext :: (s: CFSocketRef, _context: *CFSocketContext) -> void #foreign corefoundation`
- `CFSocketGetNative :: (s: CFSocketRef) -> CFSocketNativeHandle #foreign corefoundation`
- `CFSocketCreateRunLoopSource :: (allocator: CFAllocatorRef, s: CFSocketRef, order: CFIndex) -> CFRunLoopSourceRef #foreign corefoundation`
- `CFSocketGetSocketFlags :: (s: CFSocketRef) -> CFOptionFlags #foreign corefoundation`
- `CFSocketSetSocketFlags :: (s: CFSocketRef, flags: CFOptionFlags) -> void #foreign corefoundation`
- `CFSocketDisableCallBacks :: (s: CFSocketRef, callBackTypes: CFOptionFlags) -> void #foreign corefoundation`
- `CFSocketEnableCallBacks :: (s: CFSocketRef, callBackTypes: CFOptionFlags) -> void #foreign corefoundation`
- `CFSocketSendData :: (s: CFSocketRef, address: CFDataRef, data: CFDataRef, timeout: CFTimeInterval) -> CFSocketError #foreign corefoundation`
- `CFSocketRegisterValue :: (nameServerSignature: *CFSocketSignature, timeout: CFTimeInterval, name: CFStringRef, value: CFPropertyListRef) -> CFSocketError #foreign corefoundation`
- `CFSocketCopyRegisteredValue :: (nameServerSignature: *CFSocketSignature, timeout: CFTimeInterval, name: CFStringRef, value: *CFPropertyListRef, nameServerAddress: *CFDataRef) -> CFSocketError #foreign corefoundation`
- `CFSocketRegisterSocketSignature :: (nameServerSignature: *CFSocketSignature, timeout: CFTimeInterval, name: CFStringRef, signature: *CFSocketSignature) -> CFSocketError #foreign corefoundation`
- `CFSocketCopyRegisteredSocketSignature :: (nameServerSignature: *CFSocketSignature, timeout: CFTimeInterval, name: CFStringRef, signature: *CFSocketSignature, nameServerAddress: *CFDataRef) -> CFSocketError #foreign corefoundation`
- `CFSocketUnregister :: (nameServerSignature: *CFSocketSignature, timeout: CFTimeInterval, name: CFStringRef) -> CFSocketError #foreign corefoundation`
- `CFSocketSetDefaultNameRegistryPortNumber :: (port: u16) -> void #foreign corefoundation`
- `CFSocketGetDefaultNameRegistryPortNumber :: () -> u16 #foreign corefoundation`
- `CFStreamPropertyKey :: CFStringRef`
- `CFStreamStatus :: enum CFIndex`
- `CFStreamEventType :: CFOptionFlags`
- `kCFStreamEvent :: enum u32`
- `CFStreamClientContext :: struct`
- `__CFReadStream :: struct`
- `CFReadStreamRef :: *__CFReadStream`
- `__CFWriteStream :: struct`
- `CFWriteStreamRef :: *__CFWriteStream`
- `CFReadStreamClientCallBack :: #type (stream: CFReadStreamRef, type: CFStreamEventType, clientCallBackInfo: *void) -> void #c_call`
- `CFWriteStreamClientCallBack :: #type (stream: CFWriteStreamRef, type: CFStreamEventType, clientCallBackInfo: *void) -> void #c_call`
- `CFReadStreamGetTypeID :: () -> CFTypeID #foreign corefoundation`
- `CFWriteStreamGetTypeID :: () -> CFTypeID #foreign corefoundation`
- `CFReadStreamCreateWithBytesNoCopy :: (alloc: CFAllocatorRef, bytes: *u8, length: CFIndex, bytesDeallocator: CFAllocatorRef) -> CFReadStreamRef #foreign corefoundation`
- `CFWriteStreamCreateWithBuffer :: (alloc: CFAllocatorRef, buffer: *u8, bufferCapacity: CFIndex) -> CFWriteStreamRef #foreign corefoundation`
- `CFWriteStreamCreateWithAllocatedBuffers :: (alloc: CFAllocatorRef, bufferAllocator: CFAllocatorRef) -> CFWriteStreamRef #foreign corefoundation`
- `CFReadStreamCreateWithFile :: (alloc: CFAllocatorRef, fileURL: CFURLRef) -> CFReadStreamRef #foreign corefoundation`
- `CFWriteStreamCreateWithFile :: (alloc: CFAllocatorRef, fileURL: CFURLRef) -> CFWriteStreamRef #foreign corefoundation`
- `CFStreamCreateBoundPair :: (alloc: CFAllocatorRef, readStream: *CFReadStreamRef, writeStream: *CFWriteStreamRef, transferBufferSize: CFIndex) -> void #foreign corefoundation`
- `CFStreamCreatePairWithSocket :: (alloc: CFAllocatorRef, sock: CFSocketNativeHandle, readStream: *CFReadStreamRef, writeStream: *CFWriteStreamRef) -> void #foreign corefoundation`
- `CFStreamCreatePairWithSocketToHost :: (alloc: CFAllocatorRef, host: CFStringRef, port: u32, readStream: *CFReadStreamRef, writeStream: *CFWriteStreamRef) -> void #foreign corefoundation`
- `CFStreamCreatePairWithPeerSocketSignature :: (alloc: CFAllocatorRef, signature: *CFSocketSignature, readStream: *CFReadStreamRef, writeStream: *CFWriteStreamRef) -> void #foreign corefoundation`
- `CFReadStreamGetStatus :: (stream: CFReadStreamRef) -> CFStreamStatus #foreign corefoundation`
- `CFWriteStreamGetStatus :: (stream: CFWriteStreamRef) -> CFStreamStatus #foreign corefoundation`
- `CFReadStreamCopyError :: (stream: CFReadStreamRef) -> CFErrorRef #foreign corefoundation`
- `CFWriteStreamCopyError :: (stream: CFWriteStreamRef) -> CFErrorRef #foreign corefoundation`
- `CFReadStreamOpen :: (stream: CFReadStreamRef) -> Boolean #foreign corefoundation`
- `CFWriteStreamOpen :: (stream: CFWriteStreamRef) -> Boolean #foreign corefoundation`
- `CFReadStreamClose :: (stream: CFReadStreamRef) -> void #foreign corefoundation`
- `CFWriteStreamClose :: (stream: CFWriteStreamRef) -> void #foreign corefoundation`
- `CFReadStreamHasBytesAvailable :: (stream: CFReadStreamRef) -> Boolean #foreign corefoundation`
- `CFReadStreamRead :: (stream: CFReadStreamRef, buffer: *u8, bufferLength: CFIndex) -> CFIndex #foreign corefoundation`
- `CFReadStreamGetBuffer :: (stream: CFReadStreamRef, maxBytesToRead: CFIndex, numBytesRead: *CFIndex) -> *u8 #foreign corefoundation`
- `CFWriteStreamCanAcceptBytes :: (stream: CFWriteStreamRef) -> Boolean #foreign corefoundation`
- `CFWriteStreamWrite :: (stream: CFWriteStreamRef, buffer: *u8, bufferLength: CFIndex) -> CFIndex #foreign corefoundation`
- `CFReadStreamCopyProperty :: (stream: CFReadStreamRef, propertyName: CFStreamPropertyKey) -> CFPropertyListRef #foreign corefoundation`
- `CFWriteStreamCopyProperty :: (stream: CFWriteStreamRef, propertyName: CFStreamPropertyKey) -> CFPropertyListRef #foreign corefoundation`
- `CFReadStreamSetProperty :: (stream: CFReadStreamRef, propertyName: CFStreamPropertyKey, propertyValue: CFPropertyListRef) -> Boolean #foreign corefoundation`
- `CFWriteStreamSetProperty :: (stream: CFWriteStreamRef, propertyName: CFStreamPropertyKey, propertyValue: CFPropertyListRef) -> Boolean #foreign corefoundation`
- `CFReadStreamSetClient :: (stream: CFReadStreamRef, streamEvents: CFOptionFlags, clientCB: CFReadStreamClientCallBack, clientContext: *CFStreamClientContext) -> Boolean #foreign corefoundation`
- `CFWriteStreamSetClient :: (stream: CFWriteStreamRef, streamEvents: CFOptionFlags, clientCB: CFWriteStreamClientCallBack, clientContext: *CFStreamClientContext) -> Boolean #foreign corefoundation`
- `CFReadStreamScheduleWithRunLoop :: (stream: CFReadStreamRef, runLoop: CFRunLoopRef, runLoopMode: CFRunLoopMode) -> void #foreign corefoundation`
- `CFWriteStreamScheduleWithRunLoop :: (stream: CFWriteStreamRef, runLoop: CFRunLoopRef, runLoopMode: CFRunLoopMode) -> void #foreign corefoundation`
- `CFReadStreamUnscheduleFromRunLoop :: (stream: CFReadStreamRef, runLoop: CFRunLoopRef, runLoopMode: CFRunLoopMode) -> void #foreign corefoundation`
- `CFWriteStreamUnscheduleFromRunLoop :: (stream: CFWriteStreamRef, runLoop: CFRunLoopRef, runLoopMode: CFRunLoopMode) -> void #foreign corefoundation`
- `CFReadStreamSetDispatchQueue :: (stream: CFReadStreamRef, q: dispatch_queue_t) -> void #foreign corefoundation`
- `CFWriteStreamSetDispatchQueue :: (stream: CFWriteStreamRef, q: dispatch_queue_t) -> void #foreign corefoundation`
- `CFReadStreamCopyDispatchQueue :: (stream: CFReadStreamRef) -> dispatch_queue_t #foreign corefoundation`
- `CFWriteStreamCopyDispatchQueue :: (stream: CFWriteStreamRef) -> dispatch_queue_t #foreign corefoundation`
- `CFStreamErrorDomain :: enum CFIndex`
- `CFStreamError :: struct`
- `CFReadStreamGetError :: (stream: CFReadStreamRef) -> CFStreamError #foreign corefoundation`
- `CFWriteStreamGetError :: (stream: CFWriteStreamRef) -> CFStreamError #foreign corefoundation`
- `CFPropertyListMutabilityOptions :: CFOptionFlags`
- `kCFPropertyList :: enum u32`
- `CFPropertyListCreateFromXMLData :: (allocator: CFAllocatorRef, xmlData: CFDataRef, mutabilityOption: CFOptionFlags, errorString: *CFStringRef) -> CFPropertyListRef #foreign corefoundation`
- `CFPropertyListCreateXMLData :: (allocator: CFAllocatorRef, propertyList: CFPropertyListRef) -> CFDataRef #foreign corefoundation`
- `CFPropertyListCreateDeepCopy :: (allocator: CFAllocatorRef, propertyList: CFPropertyListRef, mutabilityOption: CFOptionFlags) -> CFPropertyListRef #foreign corefoundation`
- `CFPropertyListFormat :: CFIndex`
- `kCFPropertyList_1 :: enum u32`
- `CFPropertyListIsValid :: (plist: CFPropertyListRef, format: CFPropertyListFormat) -> Boolean #foreign corefoundation`
- `CFPropertyListWriteToStream :: (propertyList: CFPropertyListRef, stream: CFWriteStreamRef, format: CFPropertyListFormat, errorString: *CFStringRef) -> CFIndex #foreign corefoundation`
- `CFPropertyListCreateFromStream :: (allocator: CFAllocatorRef, stream: CFReadStreamRef, streamLength: CFIndex, mutabilityOption: CFOptionFlags, format: *CFPropertyListFormat, errorString: *CFStringRef) -> CFPropertyListRef #foreign corefoundation`
- `kCFPropertyList_2 :: enum u32`
- `CFPropertyListCreateWithData :: (allocator: CFAllocatorRef, data: CFDataRef, options: CFOptionFlags, format: *CFPropertyListFormat, error: *CFErrorRef) -> CFPropertyListRef #foreign corefoundation`
- `CFPropertyListCreateWithStream :: (allocator: CFAllocatorRef, stream: CFReadStreamRef, streamLength: CFIndex, options: CFOptionFlags, format: *CFPropertyListFormat, error: *CFErrorRef) -> CFPropertyListRef #foreign corefoundation`
- `CFPropertyListWrite :: (propertyList: CFPropertyListRef, stream: CFWriteStreamRef, format: CFPropertyListFormat, options: CFOptionFlags, error: *CFErrorRef) -> CFIndex #foreign corefoundation`
- `CFPropertyListCreateData :: (allocator: CFAllocatorRef, propertyList: CFPropertyListRef, format: CFPropertyListFormat, options: CFOptionFlags, error: *CFErrorRef) -> CFDataRef #foreign corefoundation`
- `__CFSet :: struct`
- `CFMutableSetRef :: *__CFSet`
- `CFStringEncodings :: CFIndex`
- `kCFStringEncoding_1 :: enum u32`
- `__CFTree :: struct`
- `CFURLCreateDataAndPropertiesFromResource :: (alloc: CFAllocatorRef, url: CFURLRef, resourceData: *CFDataRef, properties: *CFDictionaryRef, desiredProperties: CFArrayRef, errorCode: *s32) -> Boolean #foreign corefoundation`
- `CFURLWriteDataAndPropertiesToResource :: (url: CFURLRef, dataToWrite: CFDataRef, propertiesToWrite: CFDictionaryRef, errorCode: *s32) -> Boolean #foreign corefoundation`
- `CFURLDestroyResource :: (url: CFURLRef, errorCode: *s32) -> Boolean #foreign corefoundation`
- `CFURLCreatePropertyFromResource :: (alloc: CFAllocatorRef, url: CFURLRef, property: CFStringRef, errorCode: *s32) -> CFPropertyListRef #foreign corefoundation`
- `CFURLError :: CFIndex`
- `kCFURL_1 :: enum s32`
- `__CFUUID :: struct`
- `CFUUIDRef :: *__CFUUID`
- `CFUUIDBytes :: struct`
- `CFUUIDGetTypeID :: () -> CFTypeID #foreign corefoundation`
- `CFUUIDCreate :: (alloc: CFAllocatorRef) -> CFUUIDRef #foreign corefoundation`
- `CFUUIDCreateWithBytes :: (alloc: CFAllocatorRef, byte0: u8, byte1: u8, byte2: u8, byte3: u8, byte4: u8, byte5: u8, byte6: u8, byte7: u8, byte8: u8, byte9: u8, byte10: u8, byte11: u8, byte12: u8, byte13: u8, byte14: u8, byte15: u8) -> CFUUIDRef #foreign corefoundation`
- `CFUUIDCreateFromString :: (alloc: CFAllocatorRef, uuidStr: CFStringRef) -> CFUUIDRef #foreign corefoundation`
- `CFUUIDCreateString :: (alloc: CFAllocatorRef, uuid: CFUUIDRef) -> CFStringRef #foreign corefoundation`
- `CFUUIDGetConstantUUIDWithBytes :: (alloc: CFAllocatorRef, byte0: u8, byte1: u8, byte2: u8, byte3: u8, byte4: u8, byte5: u8, byte6: u8, byte7: u8, byte8: u8, byte9: u8, byte10: u8, byte11: u8, byte12: u8, byte13: u8, byte14: u8, byte15: u8) -> CFUUIDRef #foreign corefoundation`
- `CFUUIDGetUUIDBytes :: (uuid: CFUUIDRef) -> CFUUIDBytes #foreign corefoundation`
- `CFUUIDCreateFromUUIDBytes :: (alloc: CFAllocatorRef, bytes: CFUUIDBytes) -> CFUUIDRef #foreign corefoundation`
- `__CFBundle :: struct`
- `CFBundleRef :: *__CFBundle`
- `CFPlugInRef :: *__CFBundle`
- `CFBundleGetMainBundle :: () -> CFBundleRef #foreign corefoundation`
- `CFBundleGetBundleWithIdentifier :: (bundleID: CFStringRef) -> CFBundleRef #foreign corefoundation`
- `CFBundleGetAllBundles :: () -> CFArrayRef #foreign corefoundation`
- `CFBundleGetTypeID :: () -> CFTypeID #foreign corefoundation`
- `CFBundleCreate :: (allocator: CFAllocatorRef, bundleURL: CFURLRef) -> CFBundleRef #foreign corefoundation`
- `CFBundleCreateBundlesFromDirectory :: (allocator: CFAllocatorRef, directoryURL: CFURLRef, bundleType: CFStringRef) -> CFArrayRef #foreign corefoundation`
- `CFBundleCopyBundleURL :: (bundle: CFBundleRef) -> CFURLRef #foreign corefoundation`
- `CFBundleGetValueForInfoDictionaryKey :: (bundle: CFBundleRef, key: CFStringRef) -> CFPropertyListRef #foreign corefoundation`
- `CFBundleGetInfoDictionary :: (bundle: CFBundleRef) -> CFDictionaryRef #foreign corefoundation`
- `CFBundleGetLocalInfoDictionary :: (bundle: CFBundleRef) -> CFDictionaryRef #foreign corefoundation`
- `CFBundleGetPackageInfo :: (bundle: CFBundleRef, packageType: *u32, packageCreator: *u32) -> void #foreign corefoundation`
- `CFBundleGetIdentifier :: (bundle: CFBundleRef) -> CFStringRef #foreign corefoundation`
- `CFBundleGetVersionNumber :: (bundle: CFBundleRef) -> u32 #foreign corefoundation`
- `CFBundleGetDevelopmentRegion :: (bundle: CFBundleRef) -> CFStringRef #foreign corefoundation`
- `CFBundleCopySupportFilesDirectoryURL :: (bundle: CFBundleRef) -> CFURLRef #foreign corefoundation`
- `CFBundleCopyResourcesDirectoryURL :: (bundle: CFBundleRef) -> CFURLRef #foreign corefoundation`
- `CFBundleCopyPrivateFrameworksURL :: (bundle: CFBundleRef) -> CFURLRef #foreign corefoundation`
- `CFBundleCopySharedFrameworksURL :: (bundle: CFBundleRef) -> CFURLRef #foreign corefoundation`
- `CFBundleCopySharedSupportURL :: (bundle: CFBundleRef) -> CFURLRef #foreign corefoundation`
- `CFBundleCopyBuiltInPlugInsURL :: (bundle: CFBundleRef) -> CFURLRef #foreign corefoundation`
- `CFBundleCopyInfoDictionaryInDirectory :: (bundleURL: CFURLRef) -> CFDictionaryRef #foreign corefoundation`
- `CFBundleGetPackageInfoInDirectory :: (url: CFURLRef, packageType: *u32, packageCreator: *u32) -> Boolean #foreign corefoundation`
- `CFBundleCopyResourceURL :: (bundle: CFBundleRef, resourceName: CFStringRef, resourceType: CFStringRef, subDirName: CFStringRef) -> CFURLRef #foreign corefoundation`
- `CFBundleCopyResourceURLsOfType :: (bundle: CFBundleRef, resourceType: CFStringRef, subDirName: CFStringRef) -> CFArrayRef #foreign corefoundation`
- `CFBundleCopyLocalizedString :: (bundle: CFBundleRef, key: CFStringRef, value: CFStringRef, tableName: CFStringRef) -> CFStringRef #foreign corefoundation`
- `CFBundleCopyResourceURLInDirectory :: (bundleURL: CFURLRef, resourceName: CFStringRef, resourceType: CFStringRef, subDirName: CFStringRef) -> CFURLRef #foreign corefoundation`
- `CFBundleCopyResourceURLsOfTypeInDirectory :: (bundleURL: CFURLRef, resourceType: CFStringRef, subDirName: CFStringRef) -> CFArrayRef #foreign corefoundation`
- `CFBundleCopyBundleLocalizations :: (bundle: CFBundleRef) -> CFArrayRef #foreign corefoundation`
- `CFBundleCopyPreferredLocalizationsFromArray :: (locArray: CFArrayRef) -> CFArrayRef #foreign corefoundation`
- `CFBundleCopyLocalizationsForPreferences :: (locArray: CFArrayRef, prefArray: CFArrayRef) -> CFArrayRef #foreign corefoundation`
- `CFBundleCopyResourceURLForLocalization :: (bundle: CFBundleRef, resourceName: CFStringRef, resourceType: CFStringRef, subDirName: CFStringRef, localizationName: CFStringRef) -> CFURLRef #foreign corefoundation`
- `CFBundleCopyResourceURLsOfTypeForLocalization :: (bundle: CFBundleRef, resourceType: CFStringRef, subDirName: CFStringRef, localizationName: CFStringRef) -> CFArrayRef #foreign corefoundation`
- `CFBundleCopyInfoDictionaryForURL :: (url: CFURLRef) -> CFDictionaryRef #foreign corefoundation`
- `CFBundleCopyLocalizationsForURL :: (url: CFURLRef) -> CFArrayRef #foreign corefoundation`
- `CFBundleCopyExecutableArchitecturesForURL :: (url: CFURLRef) -> CFArrayRef #foreign corefoundation`
- `CFBundleCopyExecutableURL :: (bundle: CFBundleRef) -> CFURLRef #foreign corefoundation`
- `kCFBundleExecutableArchitecture :: enum u32`
- `CFBundleCopyExecutableArchitectures :: (bundle: CFBundleRef) -> CFArrayRef #foreign corefoundation`
- `CFBundlePreflightExecutable :: (bundle: CFBundleRef, error: *CFErrorRef) -> Boolean #foreign corefoundation`
- `CFBundleLoadExecutableAndReturnError :: (bundle: CFBundleRef, error: *CFErrorRef) -> Boolean #foreign corefoundation`
- `CFBundleLoadExecutable :: (bundle: CFBundleRef) -> Boolean #foreign corefoundation`
- `CFBundleIsExecutableLoaded :: (bundle: CFBundleRef) -> Boolean #foreign corefoundation`
- `CFBundleUnloadExecutable :: (bundle: CFBundleRef) -> void #foreign corefoundation`
- `CFBundleGetFunctionPointerForName :: (bundle: CFBundleRef, functionName: CFStringRef) -> *void #foreign corefoundation`
- `CFBundleGetFunctionPointersForNames :: (bundle: CFBundleRef, functionNames: CFArrayRef, ftbl: **void) -> void #foreign corefoundation`
- `CFBundleGetDataPointerForName :: (bundle: CFBundleRef, symbolName: CFStringRef) -> *void #foreign corefoundation`
- `CFBundleGetDataPointersForNames :: (bundle: CFBundleRef, symbolNames: CFArrayRef, stbl: **void) -> void #foreign corefoundation`
- `CFBundleCopyAuxiliaryExecutableURL :: (bundle: CFBundleRef, executableName: CFStringRef) -> CFURLRef #foreign corefoundation`
- `CFBundleGetPlugIn :: (bundle: CFBundleRef) -> CFPlugInRef #foreign corefoundation`
- `CFBundleRefNum :: s32`
- `OSStatus :: s32`
- `OSType :: u32`
- `CFBundleOpenBundleResourceMap :: (bundle: CFBundleRef) -> CFBundleRefNum #foreign corefoundation`
- `CFBundleOpenBundleResourceFiles :: (bundle: CFBundleRef, refNum: *CFBundleRefNum, localizedRefNum: *CFBundleRefNum) -> s32 #foreign corefoundation`
- `CFBundleCloseBundleResourceMap :: (bundle: CFBundleRef, refNum: CFBundleRefNum) -> void #foreign corefoundation`
- `__CFMessagePort :: struct`
- `CFMessagePortRef :: *__CFMessagePort`
- `kCFMessagePort :: enum s32`
- `CFMessagePortContext :: struct`
- `CFMessagePortCallBack :: #type (local: CFMessagePortRef, msgid: s32, data: CFDataRef, info: *void) -> CFDataRef #c_call`
- `CFMessagePortInvalidationCallBack :: #type (ms: CFMessagePortRef, info: *void) -> void #c_call`
- `CFMessagePortGetTypeID :: () -> CFTypeID #foreign corefoundation`
- `CFMessagePortCreateLocal :: (allocator: CFAllocatorRef, name: CFStringRef, callout: CFMessagePortCallBack, _context: *CFMessagePortContext, shouldFreeInfo: *Boolean) -> CFMessagePortRef #foreign corefoundation`
- `CFMessagePortCreateRemote :: (allocator: CFAllocatorRef, name: CFStringRef) -> CFMessagePortRef #foreign corefoundation`
- `CFMessagePortIsRemote :: (ms: CFMessagePortRef) -> Boolean #foreign corefoundation`
- `CFMessagePortGetName :: (ms: CFMessagePortRef) -> CFStringRef #foreign corefoundation`
- `CFMessagePortSetName :: (ms: CFMessagePortRef, newName: CFStringRef) -> Boolean #foreign corefoundation`
- `CFMessagePortGetContext :: (ms: CFMessagePortRef, _context: *CFMessagePortContext) -> void #foreign corefoundation`
- `CFMessagePortInvalidate :: (ms: CFMessagePortRef) -> void #foreign corefoundation`
- `CFMessagePortIsValid :: (ms: CFMessagePortRef) -> Boolean #foreign corefoundation`
- `CFMessagePortGetInvalidationCallBack :: (ms: CFMessagePortRef) -> CFMessagePortInvalidationCallBack #foreign corefoundation`
- `CFMessagePortSetInvalidationCallBack :: (ms: CFMessagePortRef, callout: CFMessagePortInvalidationCallBack) -> void #foreign corefoundation`
- `CFMessagePortSendRequest :: (remote: CFMessagePortRef, msgid: s32, data: CFDataRef, sendTimeout: CFTimeInterval, rcvTimeout: CFTimeInterval, replyMode: CFStringRef, returnData: *CFDataRef) -> s32 #foreign corefoundation`
- `CFMessagePortCreateRunLoopSource :: (allocator: CFAllocatorRef, local: CFMessagePortRef, order: CFIndex) -> CFRunLoopSourceRef #foreign corefoundation`
- `CFMessagePortSetDispatchQueue :: (ms: CFMessagePortRef, queue: dispatch_queue_t) -> void #foreign corefoundation`
- `CFPlugInDynamicRegisterFunction :: #type (plugIn: CFPlugInRef) -> void #c_call`
- `CFPlugInUnloadFunction :: #type (plugIn: CFPlugInRef) -> void #c_call`
- `CFPlugInFactoryFunction :: #type (allocator: CFAllocatorRef, typeUUID: CFUUIDRef) -> *void #c_call`
- `CFPlugInGetTypeID :: () -> CFTypeID #foreign corefoundation`
- `CFPlugInCreate :: (allocator: CFAllocatorRef, plugInURL: CFURLRef) -> CFPlugInRef #foreign corefoundation`
- `CFPlugInGetBundle :: (plugIn: CFPlugInRef) -> CFBundleRef #foreign corefoundation`
- `CFPlugInSetLoadOnDemand :: (plugIn: CFPlugInRef, flag: Boolean) -> void #foreign corefoundation`
- `CFPlugInIsLoadOnDemand :: (plugIn: CFPlugInRef) -> Boolean #foreign corefoundation`
- `CFPlugInFindFactoriesForPlugInType :: (typeUUID: CFUUIDRef) -> CFArrayRef #foreign corefoundation`
- `CFPlugInFindFactoriesForPlugInTypeInPlugIn :: (typeUUID: CFUUIDRef, plugIn: CFPlugInRef) -> CFArrayRef #foreign corefoundation`
- `CFPlugInInstanceCreate :: (allocator: CFAllocatorRef, factoryUUID: CFUUIDRef, typeUUID: CFUUIDRef) -> *void #foreign corefoundation`
- `CFPlugInRegisterFactoryFunction :: (factoryUUID: CFUUIDRef, func: CFPlugInFactoryFunction) -> Boolean #foreign corefoundation`
- `CFPlugInRegisterFactoryFunctionByName :: (factoryUUID: CFUUIDRef, plugIn: CFPlugInRef, functionName: CFStringRef) -> Boolean #foreign corefoundation`
- `CFPlugInUnregisterFactory :: (factoryUUID: CFUUIDRef) -> Boolean #foreign corefoundation`
- `CFPlugInRegisterPlugInType :: (factoryUUID: CFUUIDRef, typeUUID: CFUUIDRef) -> Boolean #foreign corefoundation`
- `CFPlugInUnregisterPlugInType :: (factoryUUID: CFUUIDRef, typeUUID: CFUUIDRef) -> Boolean #foreign corefoundation`
- `CFPlugInAddInstanceForFactory :: (factoryID: CFUUIDRef) -> void #foreign corefoundation`
- `CFPlugInRemoveInstanceForFactory :: (factoryID: CFUUIDRef) -> void #foreign corefoundation`
- `__CFPlugInInstance :: struct`
- `CFPlugInInstanceRef :: *__CFPlugInInstance`
- `CFPlugInInstanceGetInterfaceFunction :: #type (instance: CFPlugInInstanceRef, interfaceName: CFStringRef, ftbl: **void) -> Boolean #c_call`
- `CFPlugInInstanceDeallocateInstanceDataFunction :: #type (instanceData: *void) -> void #c_call`
- `CFPlugInInstanceGetInterfaceFunctionTable :: (instance: CFPlugInInstanceRef, interfaceName: CFStringRef, ftbl: **void) -> Boolean #foreign corefoundation`
- `CFPlugInInstanceGetFactoryName :: (instance: CFPlugInInstanceRef) -> CFStringRef #foreign corefoundation`
- `CFPlugInInstanceGetInstanceData :: (instance: CFPlugInInstanceRef) -> *void #foreign corefoundation`
- `CFPlugInInstanceGetTypeID :: () -> CFTypeID #foreign corefoundation`
- `CFPlugInInstanceCreateWithInstanceDataSize :: (allocator: CFAllocatorRef, instanceDataSize: CFIndex, deallocateInstanceFunction: CFPlugInInstanceDeallocateInstanceDataFunction, factoryName: CFStringRef, getInterfaceFunction: CFPlugInInstanceGetInterfaceFunction) -> CFPlugInInstanceRef #foreign corefoundation`
- `__CFMachPort :: struct`
- `CFMachPortRef :: *__CFMachPort`
- `CFMachPortContext :: struct`
- `CFMachPortCallBack :: #type (port: CFMachPortRef, msg: *void, size: CFIndex, info: *void) -> void #c_call`
- `CFMachPortInvalidationCallBack :: #type (port: CFMachPortRef, info: *void) -> void #c_call`
- `CFMachPortGetTypeID :: () -> CFTypeID #foreign corefoundation`
- `CFMachPortCreate :: (allocator: CFAllocatorRef, callout: CFMachPortCallBack, _context: *CFMachPortContext, shouldFreeInfo: *Boolean) -> CFMachPortRef #foreign corefoundation`
- `CFMachPortCreateWithPort :: (allocator: CFAllocatorRef, portNum: mach_port_t, callout: CFMachPortCallBack, _context: *CFMachPortContext, shouldFreeInfo: *Boolean) -> CFMachPortRef #foreign corefoundation`
- `CFMachPortGetPort :: (port: CFMachPortRef) -> mach_port_t #foreign corefoundation`
- `CFMachPortGetContext :: (port: CFMachPortRef, _context: *CFMachPortContext) -> void #foreign corefoundation`
- `CFMachPortInvalidate :: (port: CFMachPortRef) -> void #foreign corefoundation`
- `CFMachPortIsValid :: (port: CFMachPortRef) -> Boolean #foreign corefoundation`
- `CFMachPortGetInvalidationCallBack :: (port: CFMachPortRef) -> CFMachPortInvalidationCallBack #foreign corefoundation`
- `CFMachPortSetInvalidationCallBack :: (port: CFMachPortRef, callout: CFMachPortInvalidationCallBack) -> void #foreign corefoundation`
- `CFMachPortCreateRunLoopSource :: (allocator: CFAllocatorRef, port: CFMachPortRef, order: CFIndex) -> CFRunLoopSourceRef #foreign corefoundation`
- `__CFAttributedString :: struct`
- `CFAttributedStringRef :: *__CFAttributedString`
- `CFMutableAttributedStringRef :: *__CFAttributedString`
- `CFAttributedStringGetTypeID :: () -> CFTypeID #foreign corefoundation`
- `CFAttributedStringCreate :: (alloc: CFAllocatorRef, str: CFStringRef, attributes: CFDictionaryRef) -> CFAttributedStringRef #foreign corefoundation`
- `CFAttributedStringCreateWithSubstring :: (alloc: CFAllocatorRef, aStr: CFAttributedStringRef, range: CFRange) -> CFAttributedStringRef #foreign corefoundation`
- `CFAttributedStringCreateCopy :: (alloc: CFAllocatorRef, aStr: CFAttributedStringRef) -> CFAttributedStringRef #foreign corefoundation`
- `CFAttributedStringGetString :: (aStr: CFAttributedStringRef) -> CFStringRef #foreign corefoundation`
- `CFAttributedStringGetLength :: (aStr: CFAttributedStringRef) -> CFIndex #foreign corefoundation`
- `CFAttributedStringGetAttributes :: (aStr: CFAttributedStringRef, loc: CFIndex, effectiveRange: *CFRange) -> CFDictionaryRef #foreign corefoundation`
- `CFAttributedStringGetAttribute :: (aStr: CFAttributedStringRef, loc: CFIndex, attrName: CFStringRef, effectiveRange: *CFRange) -> CFPropertyListRef #foreign corefoundation`
- `CFAttributedStringGetAttributesAndLongestEffectiveRange :: (aStr: CFAttributedStringRef, loc: CFIndex, inRange: CFRange, longestEffectiveRange: *CFRange) -> CFDictionaryRef #foreign corefoundation`
- `CFAttributedStringGetAttributeAndLongestEffectiveRange :: (aStr: CFAttributedStringRef, loc: CFIndex, attrName: CFStringRef, inRange: CFRange, longestEffectiveRange: *CFRange) -> CFPropertyListRef #foreign corefoundation`
- `CFAttributedStringCreateMutableCopy :: (alloc: CFAllocatorRef, maxLength: CFIndex, aStr: CFAttributedStringRef) -> CFMutableAttributedStringRef #foreign corefoundation`
- `CFAttributedStringCreateMutable :: (alloc: CFAllocatorRef, maxLength: CFIndex) -> CFMutableAttributedStringRef #foreign corefoundation`
- `CFAttributedStringReplaceString :: (aStr: CFMutableAttributedStringRef, range: CFRange, replacement: CFStringRef) -> void #foreign corefoundation`
- `CFAttributedStringGetMutableString :: (aStr: CFMutableAttributedStringRef) -> CFMutableStringRef #foreign corefoundation`
- `CFAttributedStringSetAttributes :: (aStr: CFMutableAttributedStringRef, range: CFRange, replacement: CFDictionaryRef, clearOtherAttributes: Boolean) -> void #foreign corefoundation`
- `CFAttributedStringSetAttribute :: (aStr: CFMutableAttributedStringRef, range: CFRange, attrName: CFStringRef, value: CFPropertyListRef) -> void #foreign corefoundation`
- `CFAttributedStringRemoveAttribute :: (aStr: CFMutableAttributedStringRef, range: CFRange, attrName: CFStringRef) -> void #foreign corefoundation`
- `CFAttributedStringReplaceAttributedString :: (aStr: CFMutableAttributedStringRef, range: CFRange, replacement: CFAttributedStringRef) -> void #foreign corefoundation`
- `CFAttributedStringBeginEditing :: (aStr: CFMutableAttributedStringRef) -> void #foreign corefoundation`
- `CFAttributedStringEndEditing :: (aStr: CFMutableAttributedStringRef) -> void #foreign corefoundation`
- `__CFURLEnumerator :: struct`
- `CFURLEnumeratorRef :: *__CFURLEnumerator`
- `CFURLEnumeratorGetTypeID :: () -> CFTypeID #foreign corefoundation`
- `CFURLEnumeratorOptions :: CFOptionFlags`
- `kCFURLEnumerator :: enum u32`
- `CFURLEnumeratorCreateForDirectoryURL :: (alloc: CFAllocatorRef, directoryURL: CFURLRef, option: CFURLEnumeratorOptions, propertyKeys: CFArrayRef) -> CFURLEnumeratorRef #foreign corefoundation`
- `CFURLEnumeratorCreateForMountedVolumes :: (alloc: CFAllocatorRef, option: CFURLEnumeratorOptions, propertyKeys: CFArrayRef) -> CFURLEnumeratorRef #foreign corefoundation`
- `CFURLEnumeratorResult :: CFIndex`
- `kCFURLEnumerator_1 :: enum u32`
- `CFURLEnumeratorGetNextURL :: (enumerator: CFURLEnumeratorRef, url: *CFURLRef, error: *CFErrorRef) -> CFURLEnumeratorResult #foreign corefoundation`
- `CFURLEnumeratorSkipDescendents :: (enumerator: CFURLEnumeratorRef) -> void #foreign corefoundation`
- `CFURLEnumeratorGetDescendentLevel :: (enumerator: CFURLEnumeratorRef) -> CFIndex #foreign corefoundation`
- `CFURLEnumeratorGetSourceDidChange :: (enumerator: CFURLEnumeratorRef) -> Boolean #foreign corefoundation`
- `CFStringTokenizerCopyBestStringLanguage :: (_string: CFStringRef, range: CFRange) -> CFStringRef #foreign corefoundation`
- `__CFStringTokenizer :: struct`
- `CFStringTokenizerRef :: *__CFStringTokenizer`
- `kCFStringTokenizer :: enum u32`
- `CFStringTokenizerTokenType :: CFOptionFlags`
- `kCFStringTokenizerToken :: enum u32`
- `CFStringTokenizerGetTypeID :: () -> CFTypeID #foreign corefoundation`
- `CFStringTokenizerCreate :: (alloc: CFAllocatorRef, _string: CFStringRef, range: CFRange, options: CFOptionFlags, locale: CFLocaleRef) -> CFStringTokenizerRef #foreign corefoundation`
- `CFStringTokenizerSetString :: (tokenizer: CFStringTokenizerRef, _string: CFStringRef, range: CFRange) -> void #foreign corefoundation`
- `CFStringTokenizerGoToTokenAtIndex :: (tokenizer: CFStringTokenizerRef, index: CFIndex) -> CFStringTokenizerTokenType #foreign corefoundation`
- `CFStringTokenizerAdvanceToNextToken :: (tokenizer: CFStringTokenizerRef) -> CFStringTokenizerTokenType #foreign corefoundation`
- `CFStringTokenizerGetCurrentTokenRange :: (tokenizer: CFStringTokenizerRef) -> CFRange #foreign corefoundation`
- `CFStringTokenizerCopyCurrentTokenAttribute :: (tokenizer: CFStringTokenizerRef, attribute: CFOptionFlags) -> CFPropertyListRef #foreign corefoundation`
- `CFStringTokenizerGetCurrentSubTokens :: (tokenizer: CFStringTokenizerRef, ranges: *CFRange, maxRangeLength: CFIndex, derivedSubTokens: CFMutableArrayRef) -> CFIndex #foreign corefoundation`
- `CFFileDescriptorNativeDescriptor :: s32`
- `__CFFileDescriptor :: struct`
- `CFFileDescriptorRef :: *__CFFileDescriptor`
- `kCFFileDescriptor :: enum u32`
- `CFFileDescriptorCallBack :: #type (f: CFFileDescriptorRef, callBackTypes: CFOptionFlags, info: *void) -> void #c_call`
- `CFFileDescriptorContext :: struct`
- `CFFileDescriptorGetTypeID :: () -> CFTypeID #foreign corefoundation`
- `CFFileDescriptorCreate :: (allocator: CFAllocatorRef, fd: CFFileDescriptorNativeDescriptor, closeOnInvalidate: Boolean, callout: CFFileDescriptorCallBack, _context: *CFFileDescriptorContext) -> CFFileDescriptorRef #foreign corefoundation`
- `CFFileDescriptorGetNativeDescriptor :: (f: CFFileDescriptorRef) -> CFFileDescriptorNativeDescriptor #foreign corefoundation`
- `CFFileDescriptorGetContext :: (f: CFFileDescriptorRef, _context: *CFFileDescriptorContext) -> void #foreign corefoundation`
- `CFFileDescriptorEnableCallBacks :: (f: CFFileDescriptorRef, callBackTypes: CFOptionFlags) -> void #foreign corefoundation`
- `CFFileDescriptorDisableCallBacks :: (f: CFFileDescriptorRef, callBackTypes: CFOptionFlags) -> void #foreign corefoundation`
- `CFFileDescriptorInvalidate :: (f: CFFileDescriptorRef) -> void #foreign corefoundation`
- `CFFileDescriptorIsValid :: (f: CFFileDescriptorRef) -> Boolean #foreign corefoundation`
- `CFFileDescriptorCreateRunLoopSource :: (allocator: CFAllocatorRef, f: CFFileDescriptorRef, order: CFIndex) -> CFRunLoopSourceRef #foreign corefoundation`
- `__CFUserNotification :: struct`
- `CFUserNotificationRef :: *__CFUserNotification`
- `CFUserNotificationCallBack :: #type (userNotification: CFUserNotificationRef, responseFlags: CFOptionFlags) -> void #c_call`
- `CFUserNotificationGetTypeID :: () -> CFTypeID #foreign corefoundation`
- `CFUserNotificationCreate :: (allocator: CFAllocatorRef, timeout: CFTimeInterval, flags: CFOptionFlags, error: *s32, dictionary: CFDictionaryRef) -> CFUserNotificationRef #foreign corefoundation`
- `CFUserNotificationReceiveResponse :: (userNotification: CFUserNotificationRef, timeout: CFTimeInterval, responseFlags: *CFOptionFlags) -> s32 #foreign corefoundation`
- `CFUserNotificationGetResponseValue :: (userNotification: CFUserNotificationRef, key: CFStringRef, idx: CFIndex) -> CFStringRef #foreign corefoundation`
- `CFUserNotificationGetResponseDictionary :: (userNotification: CFUserNotificationRef) -> CFDictionaryRef #foreign corefoundation`
- `CFUserNotificationUpdate :: (userNotification: CFUserNotificationRef, timeout: CFTimeInterval, flags: CFOptionFlags, dictionary: CFDictionaryRef) -> s32 #foreign corefoundation`
- `CFUserNotificationCancel :: (userNotification: CFUserNotificationRef) -> s32 #foreign corefoundation`
- `CFUserNotificationCreateRunLoopSource :: (allocator: CFAllocatorRef, userNotification: CFUserNotificationRef, callout: CFUserNotificationCallBack, order: CFIndex) -> CFRunLoopSourceRef #foreign corefoundation`
- `CFUserNotificationDisplayNotice :: (timeout: CFTimeInterval, flags: CFOptionFlags, iconURL: CFURLRef, soundURL: CFURLRef, localizationURL: CFURLRef, alertHeader: CFStringRef, alertMessage: CFStringRef, defaultButtonTitle: CFStringRef) -> s32 #foreign corefoundation`
- `CFUserNotificationDisplayAlert :: (timeout: CFTimeInterval, flags: CFOptionFlags, iconURL: CFURLRef, soundURL: CFURLRef, localizationURL: CFURLRef, alertHeader: CFStringRef, alertMessage: CFStringRef, defaultButtonTitle: CFStringRef, alternateButtonTitle: CFStringRef, otherButtonTitle: CFStringRef, responseFlags: *CFOptionFlags) -> s32 #foreign corefoundation`
- `kCFUserNotification :: enum u32`
- `kCFUserNotification_1 :: enum u32`
- `kCFUserNotification_2 :: enum u32`
- `anon_enum_8 :: enum u32`
- `USER_ADDR_NULL :: cast(user_addr_t) 0`
- `__mbstate_t :: struct`
- `register_t :: s64`
- `user_addr_t :: u64`
- `user_size_t :: u64`
- `user_ssize_t :: s64`
- `user_long_t :: s64`
- `user_ulong_t :: u64`
- `user_time_t :: s64`
- `user_off_t :: s64`
- `syscall_arg_t :: u64`
- `mach_task_self :: () -> mach_port_t #foreign libc`
- `kern_return_t :: KERN`
- `NDR_PROTOCOL_2_0 :: 0`
- `NDR_INT_BIG_ENDIAN :: 0`
- `NDR_INT_LITTLE_ENDIAN :: 1`
- `NDR_FLOAT_IEEE :: 0`
- `NDR_FLOAT_VAX :: 1`
- `NDR_FLOAT_CRAY :: 2`
- `NDR_FLOAT_IBM :: 3`
- `NDR_CHAR_ASCII :: 0`
- `NDR_CHAR_EBCDIC :: 1`
- `__NDR_convert__ :: 0`
- `__NDR_convert__int_rep__ :: __NDR_convert__`
- `__NDR_convert__char_rep__ :: 0`
- `__NDR_convert__float_rep__ :: 0`
- `KERN :: enum u32`
- `MACH_PORT_NULL :: 0`
- `MACH_PORT_DEAD :: cast,trunc(mach_port_name_t) ~0`
- `MACH_PORT_RIGHT_SEND :: cast(mach_port_right_t) 0`
- `MACH_PORT_RIGHT_RECEIVE :: cast(mach_port_right_t) 1`
- `MACH_PORT_RIGHT_SEND_ONCE :: cast(mach_port_right_t) 2`
- `MACH_PORT_RIGHT_PORT_SET :: cast(mach_port_right_t) 3`
- `MACH_PORT_RIGHT_DEAD_NAME :: cast(mach_port_right_t) 4`
- `MACH_PORT_RIGHT_LABELH :: cast(mach_port_right_t) 5`
- `MACH_PORT_RIGHT_NUMBER :: cast(mach_port_right_t) 6`
- `MACH_PORT_TYPE_NONE :: cast(mach_port_type_t) 0`
- `MACH_PORT_TYPE_DNREQUEST :: 0x80000000`
- `MACH_PORT_TYPE_SPREQUEST :: 0x40000000`
- `MACH_PORT_TYPE_SPREQUEST_DELAYED :: 0x20000000`
- `MACH_PORT_SRIGHTS_NONE :: 0`
- `MACH_PORT_SRIGHTS_PRESENT :: 1`
- `MACH_PORT_QLIMIT_ZERO :: 0`
- `MACH_PORT_QLIMIT_BASIC :: 5`
- `MACH_PORT_QLIMIT_SMALL :: 16`
- `MACH_PORT_QLIMIT_LARGE :: 1024`
- `MACH_PORT_QLIMIT_KERNEL :: 65534`
- `MACH_PORT_QLIMIT_MIN :: MACH_PORT_QLIMIT_ZERO`
- `MACH_PORT_QLIMIT_DEFAULT :: MACH_PORT_QLIMIT_BASIC`
- `MACH_PORT_QLIMIT_MAX :: MACH_PORT_QLIMIT_LARGE`
- `MACH_PORT_STATUS_FLAG_TEMPOWNER :: 0x01`
- `MACH_PORT_STATUS_FLAG_GUARDED :: 0x02`
- `MACH_PORT_STATUS_FLAG_STRICT_GUARD :: 0x04`
- `MACH_PORT_STATUS_FLAG_IMP_DONATION :: 0x08`
- `MACH_PORT_STATUS_FLAG_REVIVE :: 0x10`
- `MACH_PORT_STATUS_FLAG_TASKPTR :: 0x20`
- `MACH_PORT_LIMITS_INFO :: 1`
- `MACH_PORT_RECEIVE_STATUS :: 2`
- `MACH_PORT_DNREQUESTS_SIZE :: 3`
- `MACH_PORT_TEMPOWNER :: 4`
- `MACH_PORT_IMPORTANCE_RECEIVER :: 5`
- `MACH_PORT_DENAP_RECEIVER :: 6`
- `MACH_PORT_INFO_EXT :: 7`
- `MACH_PORT_DNREQUESTS_SIZE_COUNT :: 1`
- `MPO_CONTEXT_AS_GUARD :: 0x01`
- `MPO_QLIMIT :: 0x02`
- `MPO_TEMPOWNER :: 0x04`
- `MPO_IMPORTANCE_RECEIVER :: 0x08`
- `MPO_INSERT_SEND_RIGHT :: 0x10`
- `MPO_STRICT :: 0x20`
- `MPO_DENAP_RECEIVER :: 0x40`
- `GUARD_TYPE_MACH_PORT :: 0x1`
- `MACH_MSG_TIMEOUT_NONE :: cast(mach_msg_timeout_t) 0`
- `MACH_MSGH_BITS_ZERO :: 0x00000000`
- `MACH_MSGH_BITS_REMOTE_MASK :: 0x0000001f`
- `MACH_MSGH_BITS_LOCAL_MASK :: 0x00001f00`
- `MACH_MSGH_BITS_VOUCHER_MASK :: 0x001f0000`
- `MACH_MSGH_BITS_PORTS_MASK :: MACH_MSGH_BITS_REMOTE_MASK | MACH_MSGH_BITS_LOCAL_MASK | MACH_MSGH_BITS_VOUCHER_MASK`
- `MACH_MSGH_BITS_COMPLEX :: 0x80000000`
- `MACH_MSGH_BITS_USER :: 0x801f1f1f`
- `MACH_MSGH_BITS_RAISEIMP :: 0x20000000`
- `MACH_MSGH_BITS_DENAP :: MACH_MSGH_BITS_RAISEIMP`
- `MACH_MSGH_BITS_IMPHOLDASRT :: 0x10000000`
- `MACH_MSGH_BITS_DENAPHOLDASRT :: MACH_MSGH_BITS_IMPHOLDASRT`
- `MACH_MSGH_BITS_CIRCULAR :: 0x10000000`
- `MACH_MSGH_BITS_USED :: 0xb01f1f1f`
- `MACH_MSG_SIZE_NULL :: cast(*mach_msg_size_t) 0`
- `MACH_MSG_PRIORITY_UNSPECIFIED :: cast(mach_msg_priority_t) 0`
- `MACH_MSG_TYPE_MOVE_RECEIVE :: 16`
- `MACH_MSG_TYPE_MOVE_SEND :: 17`
- `MACH_MSG_TYPE_MOVE_SEND_ONCE :: 18`
- `MACH_MSG_TYPE_COPY_SEND :: 19`
- `MACH_MSG_TYPE_MAKE_SEND :: 20`
- `MACH_MSG_TYPE_MAKE_SEND_ONCE :: 21`
- `MACH_MSG_TYPE_COPY_RECEIVE :: 22`
- `MACH_MSG_TYPE_DISPOSE_RECEIVE :: 24`
- `MACH_MSG_TYPE_DISPOSE_SEND :: 25`
- `MACH_MSG_TYPE_DISPOSE_SEND_ONCE :: 26`
- `MACH_MSG_PHYSICAL_COPY :: 0`
- `MACH_MSG_VIRTUAL_COPY :: 1`
- `MACH_MSG_ALLOCATE :: 2`
- `MACH_MSG_OVERWRITE :: 3`
- `MACH_MSG_PORT_DESCRIPTOR :: 0`
- `MACH_MSG_OOL_DESCRIPTOR :: 1`
- `MACH_MSG_OOL_PORTS_DESCRIPTOR :: 2`
- `MACH_MSG_OOL_VOLATILE_DESCRIPTOR :: 3`
- `MACH_MSG_BODY_NULL :: cast(*mach_msg_body_t) 0`
- `MACH_MSG_DESCRIPTOR_NULL :: cast(*mach_msg_descriptor_t) 0`
- `MACH_MSG_NULL :: cast(*mach_msg_header_t) 0`
- `MACH_MSG_TRAILER_FORMAT_0 :: 0`
- `MACH_MSG_SIZE_MAX :: cast,trunc(mach_msg_size_t) ~0`
- `MACH_MSG_SIZE_RELIABLE :: cast(mach_msg_size_t) 256 * 1024`
- `MACH_MSGH_KIND_NORMAL :: 0x00000000`
- `MACH_MSGH_KIND_NOTIFICATION :: 0x00000001`
- `mach_msg_kind_t :: mach_port_seqno_t`
- `MACH_MSG_TYPE_PORT_NONE :: 0`
- `MACH_MSG_TYPE_PORT_NAME :: 15`
- `MACH_MSG_TYPE_PORT_RECEIVE :: MACH_MSG_TYPE_MOVE_RECEIVE`
- `MACH_MSG_TYPE_PORT_SEND :: MACH_MSG_TYPE_MOVE_SEND`
- `MACH_MSG_TYPE_PORT_SEND_ONCE :: MACH_MSG_TYPE_MOVE_SEND_ONCE`
- `MACH_MSG_TYPE_LAST :: 22`
- `MACH_MSG_TYPE_POLYMORPHIC :: cast,trunc(mach_msg_type_name_t) -1`
- `MACH_MSG_OPTION_NONE :: 0x00000000`
- `MACH_SEND_MSG :: 0x00000001`
- `MACH_RCV_MSG :: 0x00000002`
- `MACH_RCV_LARGE :: 0x00000004`
- `MACH_RCV_LARGE_IDENTITY :: 0x00000008`
- `MACH_SEND_TIMEOUT :: 0x00000010`
- `MACH_SEND_OVERRIDE :: 0x00000020`
- `MACH_SEND_INTERRUPT :: 0x00000040`
- `MACH_SEND_NOTIFY :: 0x00000080`
- `MACH_SEND_ALWAYS :: 0x00010000`
- `MACH_SEND_TRAILER :: 0x00020000`
- `MACH_SEND_NOIMPORTANCE :: 0x00040000`
- `MACH_SEND_NODENAP :: MACH_SEND_NOIMPORTANCE`
- `MACH_SEND_IMPORTANCE :: 0x00080000`
- `MACH_SEND_SYNC_OVERRIDE :: 0x00100000`
- `MACH_RCV_TIMEOUT :: 0x00000100`
- `MACH_RCV_NOTIFY :: 0x00000200`
- `MACH_RCV_INTERRUPT :: 0x00000400`
- `MACH_RCV_VOUCHER :: 0x00000800`
- `MACH_RCV_OVERWRITE :: 0x00001000`
- `MACH_RCV_SYNC_WAIT :: 0x00004000`
- `MACH_RCV_TRAILER_NULL :: 0`
- `MACH_RCV_TRAILER_SEQNO :: 1`
- `MACH_RCV_TRAILER_SENDER :: 2`
- `MACH_RCV_TRAILER_AUDIT :: 3`
- `MACH_RCV_TRAILER_CTX :: 4`
- `MACH_RCV_TRAILER_AV :: 7`
- `MACH_RCV_TRAILER_LABELS :: 8`
- `MACH_RCV_TRAILER_MASK :: (0xf << 24)`
- `MACH_MSG_SUCCESS :: 0x00000000`
- `MACH_MSG_MASK :: 0x00003e00`
- `MACH_MSG_IPC_SPACE :: 0x00002000`
- `MACH_MSG_VM_SPACE :: 0x00001000`
- `MACH_MSG_IPC_KERNEL :: 0x00000800`
- `MACH_MSG_VM_KERNEL :: 0x00000400`
- `MACH_SEND_IN_PROGRESS :: 0x10000001`
- `MACH_SEND_INVALID_DATA :: 0x10000002`
- `MACH_SEND_INVALID_DEST :: 0x10000003`
- `MACH_SEND_TIMED_OUT :: 0x10000004`
- `MACH_SEND_INVALID_VOUCHER :: 0x10000005`
- `MACH_SEND_INTERRUPTED :: 0x10000007`
- `MACH_SEND_MSG_TOO_SMALL :: 0x10000008`
- `MACH_SEND_INVALID_REPLY :: 0x10000009`
- `MACH_SEND_INVALID_RIGHT :: 0x1000000a`
- `MACH_SEND_INVALID_NOTIFY :: 0x1000000b`
- `MACH_SEND_INVALID_MEMORY :: 0x1000000c`
- `MACH_SEND_NO_BUFFER :: 0x1000000d`
- `MACH_SEND_TOO_LARGE :: 0x1000000e`
- `MACH_SEND_INVALID_TYPE :: 0x1000000f`
- `MACH_SEND_INVALID_HEADER :: 0x10000010`
- `MACH_SEND_INVALID_TRAILER :: 0x10000011`
- `MACH_SEND_INVALID_RT_OOL_SIZE :: 0x10000015`
- `MACH_RCV_IN_PROGRESS :: 0x10004001`
- `MACH_RCV_INVALID_NAME :: 0x10004002`
- `MACH_RCV_TIMED_OUT :: 0x10004003`
- `MACH_RCV_TOO_LARGE :: 0x10004004`
- `MACH_RCV_INTERRUPTED :: 0x10004005`
- `MACH_RCV_PORT_CHANGED :: 0x10004006`
- `MACH_RCV_INVALID_NOTIFY :: 0x10004007`
- `MACH_RCV_INVALID_DATA :: 0x10004008`
- `MACH_RCV_PORT_DIED :: 0x10004009`
- `MACH_RCV_IN_SET :: 0x1000400a`
- `MACH_RCV_HEADER_ERROR :: 0x1000400b`
- `MACH_RCV_BODY_ERROR :: 0x1000400c`
- `MACH_RCV_INVALID_TYPE :: 0x1000400d`
- `MACH_RCV_SCATTER_SMALL :: 0x1000400e`
- `MACH_RCV_INVALID_TRAILER :: 0x1000400f`
- `MACH_RCV_IN_PROGRESS_TIMED :: 0x10004011`
- `VM_PAGE_QUERY_PAGE_PRESENT :: 0x1`
- `VM_PAGE_QUERY_PAGE_FICTITIOUS :: 0x2`
- `VM_PAGE_QUERY_PAGE_REF :: 0x4`
- `VM_PAGE_QUERY_PAGE_DIRTY :: 0x8`
- `VM_PAGE_QUERY_PAGE_PAGED_OUT :: 0x10`
- `VM_PAGE_QUERY_PAGE_COPIED :: 0x20`
- `VM_PAGE_QUERY_PAGE_SPECULATIVE :: 0x40`
- `VM_PAGE_QUERY_PAGE_EXTERNAL :: 0x80`
- `VM_PAGE_QUERY_PAGE_CS_VALIDATED :: 0x100`
- `VM_PAGE_QUERY_PAGE_CS_TAINTED :: 0x200`
- `VM_PAGE_QUERY_PAGE_CS_NX :: 0x400`
- `VM_FLAGS_FIXED :: 0x0000`
- `VM_FLAGS_ANYWHERE :: 0x0001`
- `VM_FLAGS_PURGABLE :: 0x0002`
- `VM_FLAGS_4GB_CHUNK :: 0x0004`
- `VM_FLAGS_RANDOM_ADDR :: 0x0008`
- `VM_FLAGS_NO_CACHE :: 0x0010`
- `VM_FLAGS_RESILIENT_CODESIGN :: 0x0020`
- `VM_FLAGS_RESILIENT_MEDIA :: 0x0040`
- `VM_FLAGS_OVERWRITE :: 0x4000`
- `VM_FLAGS_SUPERPAGE_MASK :: 0x70000`
- `VM_FLAGS_RETURN_DATA_ADDR :: 0x100000`
- `VM_FLAGS_RETURN_4K_DATA_ADDR :: 0x800000`
- `VM_FLAGS_ALIAS_MASK :: 0xFF000000`
- `VM_FLAGS_USER_ALLOCATE :: VM_FLAGS_FIXED | VM_FLAGS_ANYWHERE | VM_FLAGS_PURGABLE | VM_FLAGS_4GB_CHUNK | VM_FLAGS_RANDOM_ADDR | VM_FLAGS_NO_CACHE | VM_FLAGS_OVERWRITE | VM_FLAGS_SUPERPAGE_MASK | VM_FLAGS_ALIAS_MASK`
- `VM_FLAGS_USER_MAP :: VM_FLAGS_USER_ALLOCATE | VM_FLAGS_RETURN_4K_DATA_ADDR | VM_FLAGS_RETURN_DATA_ADDR`
- `VM_FLAGS_USER_REMAP :: VM_FLAGS_FIXED | VM_FLAGS_ANYWHERE | VM_FLAGS_RANDOM_ADDR | VM_FLAGS_OVERWRITE| VM_FLAGS_RETURN_DATA_ADDR | VM_FLAGS_RESILIENT_CODESIGN`
- `VM_FLAGS_SUPERPAGE_SHIFT :: 16`
- `SUPERPAGE_NONE :: 0`
- `SUPERPAGE_SIZE_ANY :: 1`
- `VM_FLAGS_SUPERPAGE_NONE :: SUPERPAGE_NONE << VM_FLAGS_SUPERPAGE_SHIFT`
- `VM_FLAGS_SUPERPAGE_SIZE_ANY :: SUPERPAGE_SIZE_ANY << VM_FLAGS_SUPERPAGE_SHIFT`
- `SUPERPAGE_SIZE_2MB :: 2`
- `VM_FLAGS_SUPERPAGE_SIZE_2MB :: SUPERPAGE_SIZE_2MB<<VM_FLAGS_SUPERPAGE_SHIFT`
- `VM_MEMORY_MALLOC :: 1`
- `VM_MEMORY_MALLOC_SMALL :: 2`
- `VM_MEMORY_MALLOC_LARGE :: 3`
- `VM_MEMORY_MALLOC_HUGE :: 4`
- `VM_MEMORY_SBRK :: 5`
- `VM_MEMORY_REALLOC :: 6`
- `VM_MEMORY_MALLOC_TINY :: 7`
- `VM_MEMORY_MALLOC_LARGE_REUSABLE :: 8`
- `VM_MEMORY_MALLOC_LARGE_REUSED :: 9`
- `VM_MEMORY_ANALYSIS_TOOL :: 10`
- `VM_MEMORY_MALLOC_NANO :: 11`
- `VM_MEMORY_MACH_MSG :: 20`
- `VM_MEMORY_IOKIT :: 21`
- `VM_MEMORY_STACK :: 30`
- `VM_MEMORY_GUARD :: 31`
- `VM_MEMORY_SHARED_PMAP :: 32`
- `VM_MEMORY_DYLIB :: 33`
- `VM_MEMORY_OBJC_DISPATCHERS :: 34`
- `VM_MEMORY_UNSHARED_PMAP :: 35`
- `VM_MEMORY_APPKIT :: 40`
- `VM_MEMORY_FOUNDATION :: 41`
- `VM_MEMORY_COREGRAPHICS :: 42`
- `VM_MEMORY_CORESERVICES :: 43`
- `VM_MEMORY_CARBON :: VM_MEMORY_CORESERVICES`
- `VM_MEMORY_JAVA :: 44`
- `VM_MEMORY_COREDATA :: 45`
- `VM_MEMORY_COREDATA_OBJECTIDS :: 46`
- `VM_MEMORY_ATS :: 50`
- `VM_MEMORY_LAYERKIT :: 51`
- `VM_MEMORY_CGIMAGE :: 52`
- `VM_MEMORY_TCMALLOC :: 53`
- `VM_MEMORY_COREGRAPHICS_DATA :: 54`
- `VM_MEMORY_COREGRAPHICS_SHARED :: 55`
- `VM_MEMORY_COREGRAPHICS_FRAMEBUFFERS :: 56`
- `VM_MEMORY_COREGRAPHICS_BACKINGSTORES :: 57`
- `VM_MEMORY_COREGRAPHICS_XALLOC :: 58`
- `VM_MEMORY_COREGRAPHICS_MISC :: VM_MEMORY_COREGRAPHICS`
- `VM_MEMORY_DYLD :: 60`
- `VM_MEMORY_DYLD_MALLOC :: 61`
- `VM_MEMORY_SQLITE :: 62`
- `VM_MEMORY_JAVASCRIPT_CORE :: 63`
- `VM_MEMORY_WEBASSEMBLY :: VM_MEMORY_JAVASCRIPT_CORE`
- `VM_MEMORY_JAVASCRIPT_JIT_EXECUTABLE_ALLOCATOR :: 64`
- `VM_MEMORY_JAVASCRIPT_JIT_REGISTER_FILE :: 65`
- `VM_MEMORY_GLSL :: 66`
- `VM_MEMORY_OPENCL :: 67`
- `VM_MEMORY_COREIMAGE :: 68`
- `VM_MEMORY_WEBCORE_PURGEABLE_BUFFERS :: 69`
- `VM_MEMORY_IMAGEIO :: 70`
- `VM_MEMORY_COREPROFILE :: 71`
- `VM_MEMORY_ASSETSD :: 72`
- `VM_MEMORY_OS_ALLOC_ONCE :: 73`
- `VM_MEMORY_LIBDISPATCH :: 74`
- `VM_MEMORY_ACCELERATE :: 75`
- `VM_MEMORY_COREUI :: 76`
- `VM_MEMORY_COREUIFILE :: 77`
- `VM_MEMORY_GENEALOGY :: 78`
- `VM_MEMORY_RAWCAMERA :: 79`
- `VM_MEMORY_CORPSEINFO :: 80`
- `VM_MEMORY_ASL :: 81`
- `VM_MEMORY_SWIFT_RUNTIME :: 82`
- `VM_MEMORY_SWIFT_METADATA :: 83`
- `VM_MEMORY_DHMM :: 84`
- `VM_MEMORY_SCENEKIT :: 86`
- `VM_MEMORY_SKYWALK :: 87`
- `VM_MEMORY_IOSURFACE :: 88`
- `VM_MEMORY_LIBNETWORK :: 89`
- `VM_MEMORY_AUDIO :: 90`
- `VM_MEMORY_VIDEOBITSTREAM :: 91`
- `VM_MEMORY_APPLICATION_SPECIFIC_1 :: 240`
- `VM_MEMORY_APPLICATION_SPECIFIC_16 :: 255`
- `CPU_STATE_MAX :: 4`
- `CPU_STATE_USER :: 0`
- `CPU_STATE_SYSTEM :: 1`
- `CPU_STATE_IDLE :: 2`
- `CPU_STATE_NICE :: 3`
- `CPU_ARCH_MASK :: 0xff000000`
- `CPU_ARCH_ABI64 :: 0x01000000`
- `CPU_TYPE_ANY :: cast,trunc(cpu_type_t) -1`
- `CPU_TYPE_VAX :: cast(cpu_type_t) 1`
- `CPU_TYPE_MC680x0 :: cast(cpu_type_t) 6`
- `CPU_TYPE_X86 :: cast(cpu_type_t) 7`
- `CPU_TYPE_I386 :: CPU_TYPE_X86`
- `CPU_TYPE_X86_64 :: CPU_TYPE_X86 | CPU_ARCH_ABI64`
- `CPU_TYPE_MC98000 :: cast(cpu_type_t) 10`
- `CPU_TYPE_HPPA :: cast(cpu_type_t) 11`
- `CPU_TYPE_ARM :: cast(cpu_type_t) 12`
- `CPU_TYPE_ARM64 :: CPU_TYPE_ARM | CPU_ARCH_ABI64`
- `CPU_TYPE_MC88000 :: cast(cpu_type_t) 13`
- `CPU_TYPE_SPARC :: cast(cpu_type_t) 14`
- `CPU_TYPE_I860 :: cast(cpu_type_t) 15`
- `CPU_TYPE_POWERPC :: cast(cpu_type_t) 18`
- `CPU_TYPE_POWERPC64 :: CPU_TYPE_POWERPC | CPU_ARCH_ABI64`
- `CPU_SUBTYPE_MASK :: 0xff000000`
- `CPU_SUBTYPE_LIB64 :: 0x80000000`
- `CPU_SUBTYPE_MULTIPLE :: cast,trunc(cpu_subtype_t) -1`
- `CPU_SUBTYPE_LITTLE_ENDIAN :: cast(cpu_subtype_t) 0`
- `CPU_SUBTYPE_BIG_ENDIAN :: cast(cpu_subtype_t) 1`
- `CPU_THREADTYPE_NONE :: cast(cpu_threadtype_t) 0`
- `CPU_SUBTYPE_VAX_ALL :: cast(cpu_subtype_t) 0`
- `CPU_SUBTYPE_VAX780 :: cast(cpu_subtype_t) 1`
- `CPU_SUBTYPE_VAX785 :: cast(cpu_subtype_t) 2`
- `CPU_SUBTYPE_VAX750 :: cast(cpu_subtype_t) 3`
- `CPU_SUBTYPE_VAX730 :: cast(cpu_subtype_t) 4`
- `CPU_SUBTYPE_UVAXI :: cast(cpu_subtype_t) 5`
- `CPU_SUBTYPE_UVAXII :: cast(cpu_subtype_t) 6`
- `CPU_SUBTYPE_VAX8200 :: cast(cpu_subtype_t) 7`
- `CPU_SUBTYPE_VAX8500 :: cast(cpu_subtype_t) 8`
- `CPU_SUBTYPE_VAX8600 :: cast(cpu_subtype_t) 9`
- `CPU_SUBTYPE_VAX8650 :: cast(cpu_subtype_t) 10`
- `CPU_SUBTYPE_VAX8800 :: cast(cpu_subtype_t) 11`
- `CPU_SUBTYPE_UVAXIII :: cast(cpu_subtype_t) 12`
- `CPU_SUBTYPE_MC680x0_ALL :: cast(cpu_subtype_t) 1`
- `CPU_SUBTYPE_MC68030 :: cast(cpu_subtype_t) 1`
- `CPU_SUBTYPE_MC68040 :: cast(cpu_subtype_t) 2`
- `CPU_SUBTYPE_MC68030_ONLY :: cast(cpu_subtype_t) 3`
- `CPU_SUBTYPE_INTEL_FAMILY_MAX :: 15`
- `CPU_SUBTYPE_INTEL_MODEL_ALL :: 0`
- `CPU_SUBTYPE_X86_ALL :: cast(cpu_subtype_t) 3`
- `CPU_SUBTYPE_X86_64_ALL :: cast(cpu_subtype_t) 3`
- `CPU_SUBTYPE_X86_ARCH1 :: cast(cpu_subtype_t) 4`
- `CPU_SUBTYPE_X86_64_H :: cast(cpu_subtype_t) 8`
- `CPU_THREADTYPE_INTEL_HTT :: cast(cpu_threadtype_t) 1`
- `CPU_SUBTYPE_MIPS_ALL :: cast(cpu_subtype_t) 0`
- `CPU_SUBTYPE_MIPS_R2300 :: cast(cpu_subtype_t) 1`
- `CPU_SUBTYPE_MIPS_R2600 :: cast(cpu_subtype_t) 2`
- `CPU_SUBTYPE_MIPS_R2800 :: cast(cpu_subtype_t) 3`
- `CPU_SUBTYPE_MIPS_R2000a :: cast(cpu_subtype_t) 4`
- `CPU_SUBTYPE_MIPS_R2000 :: cast(cpu_subtype_t) 5`
- `CPU_SUBTYPE_MIPS_R3000a :: cast(cpu_subtype_t) 6`
- `CPU_SUBTYPE_MIPS_R3000 :: cast(cpu_subtype_t) 7`
- `CPU_SUBTYPE_MC98000_ALL :: cast(cpu_subtype_t) 0`
- `CPU_SUBTYPE_MC98601 :: cast(cpu_subtype_t) 1`
- `CPU_SUBTYPE_HPPA_ALL :: cast(cpu_subtype_t) 0`
- `CPU_SUBTYPE_HPPA_7100 :: cast(cpu_subtype_t) 0`
- `CPU_SUBTYPE_HPPA_7100LC :: cast(cpu_subtype_t) 1`
- `CPU_SUBTYPE_MC88000_ALL :: cast(cpu_subtype_t) 0`
- `CPU_SUBTYPE_MC88100 :: cast(cpu_subtype_t) 1`
- `CPU_SUBTYPE_MC88110 :: cast(cpu_subtype_t) 2`
- `CPU_SUBTYPE_SPARC_ALL :: cast(cpu_subtype_t) 0`
- `CPU_SUBTYPE_I860_ALL :: cast(cpu_subtype_t) 0`
- `CPU_SUBTYPE_I860_860 :: cast(cpu_subtype_t) 1`
- `CPU_SUBTYPE_POWERPC_ALL :: cast(cpu_subtype_t) 0`
- `CPU_SUBTYPE_POWERPC_601 :: cast(cpu_subtype_t) 1`
- `CPU_SUBTYPE_POWERPC_602 :: cast(cpu_subtype_t) 2`
- `CPU_SUBTYPE_POWERPC_603 :: cast(cpu_subtype_t) 3`
- `CPU_SUBTYPE_POWERPC_603e :: cast(cpu_subtype_t) 4`
- `CPU_SUBTYPE_POWERPC_603ev :: cast(cpu_subtype_t) 5`
- `CPU_SUBTYPE_POWERPC_604 :: cast(cpu_subtype_t) 6`
- `CPU_SUBTYPE_POWERPC_604e :: cast(cpu_subtype_t) 7`
- `CPU_SUBTYPE_POWERPC_620 :: cast(cpu_subtype_t) 8`
- `CPU_SUBTYPE_POWERPC_750 :: cast(cpu_subtype_t) 9`
- `CPU_SUBTYPE_POWERPC_7400 :: cast(cpu_subtype_t) 10`
- `CPU_SUBTYPE_POWERPC_7450 :: cast(cpu_subtype_t) 11`
- `CPU_SUBTYPE_POWERPC_970 :: cast(cpu_subtype_t) 100`
- `CPU_SUBTYPE_ARM_ALL :: cast(cpu_subtype_t) 0`
- `CPU_SUBTYPE_ARM_V4T :: cast(cpu_subtype_t) 5`
- `CPU_SUBTYPE_ARM_V6 :: cast(cpu_subtype_t) 6`
- `CPU_SUBTYPE_ARM_V5TEJ :: cast(cpu_subtype_t) 7`
- `CPU_SUBTYPE_ARM_XSCALE :: cast(cpu_subtype_t) 8`
- `CPU_SUBTYPE_ARM_V7 :: cast(cpu_subtype_t) 9`
- `CPU_SUBTYPE_ARM_V7F :: cast(cpu_subtype_t) 10`
- `CPU_SUBTYPE_ARM_V7S :: cast(cpu_subtype_t) 11`
- `CPU_SUBTYPE_ARM_V7K :: cast(cpu_subtype_t) 12`
- `CPU_SUBTYPE_ARM_V6M :: cast(cpu_subtype_t) 14`
- `CPU_SUBTYPE_ARM_V7M :: cast(cpu_subtype_t) 15`
- `CPU_SUBTYPE_ARM_V7EM :: cast(cpu_subtype_t) 16`
- `CPU_SUBTYPE_ARM_V8 :: cast(cpu_subtype_t) 13`
- `CPU_SUBTYPE_ARM64_ALL :: cast(cpu_subtype_t) 0`
- `CPU_SUBTYPE_ARM64_V8 :: cast(cpu_subtype_t) 1`
- `CPUFAMILY_UNKNOWN :: 0`
- `CPUFAMILY_POWERPC_G3 :: 0xcee41549`
- `CPUFAMILY_POWERPC_G4 :: 0x77c184ae`
- `CPUFAMILY_POWERPC_G5 :: 0xed76d8aa`
- `CPUFAMILY_INTEL_6_13 :: 0xaa33392b`
- `CPUFAMILY_INTEL_PENRYN :: 0x78ea4fbc`
- `CPUFAMILY_INTEL_NEHALEM :: 0x6b5a4cd2`
- `CPUFAMILY_INTEL_WESTMERE :: 0x573b5eec`
- `CPUFAMILY_INTEL_SANDYBRIDGE :: 0x5490b78c`
- `CPUFAMILY_INTEL_IVYBRIDGE :: 0x1f65e835`
- `CPUFAMILY_INTEL_HASWELL :: 0x10b282dc`
- `CPUFAMILY_INTEL_BROADWELL :: 0x582ed09c`
- `CPUFAMILY_INTEL_SKYLAKE :: 0x37fc219f`
- `CPUFAMILY_INTEL_KABYLAKE :: 0x0f817246`
- `CPUFAMILY_ARM_9 :: 0xe73283ae`
- `CPUFAMILY_ARM_11 :: 0x8ff620d8`
- `CPUFAMILY_ARM_XSCALE :: 0x53b005f5`
- `CPUFAMILY_ARM_12 :: 0xbd1b0ae9`
- `CPUFAMILY_ARM_13 :: 0x0cc90e64`
- `CPUFAMILY_ARM_14 :: 0x96077ef1`
- `CPUFAMILY_ARM_15 :: 0xa8511bca`
- `CPUFAMILY_ARM_SWIFT :: 0x1e2d6381`
- `CPUFAMILY_ARM_CYCLONE :: 0x37a09642`
- `CPUFAMILY_ARM_TYPHOON :: 0x2c91a47e`
- `CPUFAMILY_ARM_TWISTER :: 0x92fb37c8`
- `CPUFAMILY_ARM_HURRICANE :: 0x67ceee93`
- `CPUFAMILY_ARM_MONSOON_MISTRAL :: 0xe81e7ef6`
- `CPUFAMILY_INTEL_6_23 :: CPUFAMILY_INTEL_PENRYN`
- `CPUFAMILY_INTEL_6_26 :: CPUFAMILY_INTEL_NEHALEM`
- `TIME_MICROS_MAX :: 1000000`
- `VM_MAP_NULL :: cast(vm_map_t) 0`
- `UPL_NULL :: cast(upl_t) 0`
- `VM_NAMED_ENTRY_NULL :: cast(vm_named_entry_t) 0`
- `EXC_BAD_ACCESS :: 1`
- `EXC_BAD_INSTRUCTION :: 2`
- `EXC_ARITHMETIC :: 3`
- `EXC_EMULATION :: 4`
- `EXC_SOFTWARE :: 5`
- `EXC_BREAKPOINT :: 6`
- `EXC_SYSCALL :: 7`
- `EXC_MACH_SYSCALL :: 8`
- `EXC_RPC_ALERT :: 9`
- `EXC_CRASH :: 10`
- `EXC_RESOURCE :: 11`
- `EXC_GUARD :: 12`
- `EXC_CORPSE_NOTIFY :: 13`
- `EXC_CORPSE_VARIANT_BIT :: 0x100`
- `EXCEPTION_DEFAULT :: 1`
- `EXCEPTION_STATE :: 2`
- `EXCEPTION_STATE_IDENTITY :: 3`
- `MACH_EXCEPTION_CODES :: 0x80000000`
- `EXC_MASK_BAD_ACCESS :: 1 << EXC_BAD_ACCESS`
- `EXC_MASK_BAD_INSTRUCTION :: 1 << EXC_BAD_INSTRUCTION`
- `EXC_MASK_ARITHMETIC :: 1 << EXC_ARITHMETIC`
- `EXC_MASK_EMULATION :: 1 << EXC_EMULATION`
- `EXC_MASK_SOFTWARE :: 1 << EXC_SOFTWARE`
- `EXC_MASK_BREAKPOINT :: 1 << EXC_BREAKPOINT`
- `EXC_MASK_SYSCALL :: 1 << EXC_SYSCALL`
- `EXC_MASK_MACH_SYSCALL :: 1 << EXC_MACH_SYSCALL`
- `EXC_MASK_RPC_ALERT :: 1 << EXC_RPC_ALERT`
- `EXC_MASK_CRASH :: 1 << EXC_CRASH`
- `EXC_MASK_RESOURCE :: 1 << EXC_RESOURCE`
- `EXC_MASK_GUARD :: 1 << EXC_GUARD`
- `EXC_MASK_CORPSE_NOTIFY :: 1 << EXC_CORPSE_NOTIFY`
- `FIRST_EXCEPTION :: 1`
- `EXC_SOFT_SIGNAL :: 0x10003`
- `EXC_MACF_MIN :: 0x20000`
- `EXC_MACF_MAX :: 0x2FFFF`
- `x86_THREAD_STATE :: 7`
- `THREAD_STATE_NONE :: 13`
- `USER_CODE_SELECTOR :: 0x0017`
- `USER_DATA_SELECTOR :: 0x001f`
- `MACHINE_THREAD_STATE :: x86_THREAD_STATE`
- `THREAD_STATE_FLAVOR_LIST :: 0`
- `THREAD_STATE_FLAVOR_LIST_NEW :: 128`
- `THREAD_STATE_FLAVOR_LIST_10_9 :: 129`
- `THREAD_STATE_FLAVOR_LIST_10_13 :: 130`
- `MACH_VOUCHER_NULL :: cast(mach_voucher_t) 0`
- `MACH_VOUCHER_NAME_NULL :: cast(mach_voucher_name_t) 0`
- `MACH_VOUCHER_NAME_ARRAY_NULL :: cast(mach_voucher_name_array_t) 0`
- `IPC_VOUCHER_NULL :: cast(ipc_voucher_t) 0`
- `MACH_VOUCHER_SELECTOR_CURRENT :: cast(mach_voucher_selector_t) 0`
- `MACH_VOUCHER_SELECTOR_EFFECTIVE :: cast(mach_voucher_selector_t) 1`
- `MACH_VOUCHER_ATTR_KEY_ALL :: cast,trunc(mach_voucher_attr_key_t) ~0`
- `MACH_VOUCHER_ATTR_KEY_NONE :: cast(mach_voucher_attr_key_t) 0`
- `MACH_VOUCHER_ATTR_KEY_ATM :: cast(mach_voucher_attr_key_t) 1`
- `MACH_VOUCHER_ATTR_KEY_IMPORTANCE :: cast(mach_voucher_attr_key_t) 2`
- `MACH_VOUCHER_ATTR_KEY_BANK :: cast(mach_voucher_attr_key_t) 3`
- `MACH_VOUCHER_ATTR_KEY_PTHPRIORITY :: cast(mach_voucher_attr_key_t) 4`
- `MACH_VOUCHER_ATTR_KEY_USER_DATA :: cast(mach_voucher_attr_key_t) 7`
- `MACH_VOUCHER_ATTR_KEY_BITS :: MACH_VOUCHER_ATTR_KEY_USER_DATA`
- `MACH_VOUCHER_ATTR_KEY_TEST :: cast(mach_voucher_attr_key_t) 8`
- `MACH_VOUCHER_ATTR_KEY_NUM_WELL_KNOWN :: MACH_VOUCHER_ATTR_KEY_TEST`
- `MACH_VOUCHER_ATTR_NOOP :: cast(mach_voucher_attr_recipe_command_t) 0`
- `MACH_VOUCHER_ATTR_COPY :: cast(mach_voucher_attr_recipe_command_t) 1`
- `MACH_VOUCHER_ATTR_REMOVE :: cast(mach_voucher_attr_recipe_command_t) 2`
- `MACH_VOUCHER_ATTR_SET_VALUE_HANDLE :: cast(mach_voucher_attr_recipe_command_t) 3`
- `MACH_VOUCHER_ATTR_AUTO_REDEEM :: cast(mach_voucher_attr_recipe_command_t) 4`
- `MACH_VOUCHER_ATTR_SEND_PREPROCESS :: cast(mach_voucher_attr_recipe_command_t) 5`
- `MACH_VOUCHER_ATTR_REDEEM :: cast(mach_voucher_attr_recipe_command_t) 10`
- `MACH_VOUCHER_ATTR_IMPORTANCE_SELF :: cast(mach_voucher_attr_recipe_command_t) 200`
- `MACH_VOUCHER_ATTR_USER_DATA_STORE :: cast(mach_voucher_attr_recipe_command_t) 211`
- `MACH_VOUCHER_ATTR_BITS_STORE :: MACH_VOUCHER_ATTR_USER_DATA_STORE`
- `MACH_VOUCHER_ATTR_TEST_STORE :: MACH_VOUCHER_ATTR_USER_DATA_STORE`
- `MACH_VOUCHER_ATTR_MAX_RAW_RECIPE_ARRAY_SIZE :: 5120`
- `MACH_VOUCHER_TRAP_STACK_LIMIT :: 256`
- `MACH_VOUCHER_ATTR_MANAGER_NULL :: cast(mach_voucher_attr_manager_t) 0`
- `MACH_VOUCHER_ATTR_CONTROL_NULL :: cast(mach_voucher_attr_control_t) 0`
- `IPC_VOUCHER_ATTR_MANAGER_NULL :: cast(ipc_voucher_attr_manager_t) 0`
- `IPC_VOUCHER_ATTR_CONTROL_NULL :: cast(ipc_voucher_attr_control_t) 0`
- `MACH_VOUCHER_ATTR_VALUE_MAX_NESTED :: cast(mach_voucher_attr_value_handle_array_size_t) 4`
- `MACH_VOUCHER_ATTR_VALUE_FLAGS_NONE :: cast(mach_voucher_attr_value_flags_t) 0`
- `MACH_VOUCHER_ATTR_VALUE_FLAGS_PERSIST :: cast(mach_voucher_attr_value_flags_t) 1`
- `MACH_VOUCHER_ATTR_CONTROL_FLAGS_NONE :: cast(mach_voucher_attr_control_flags_t) 0`
- `MACH_VOUCHER_IMPORTANCE_ATTR_ADD_EXTERNAL :: 1`
- `MACH_VOUCHER_IMPORTANCE_ATTR_DROP_EXTERNAL :: 2`
- `MACH_ACTIVITY_ID_COUNT_MAX :: 16`
- `POLICY_NULL :: 0`
- `POLICY_TIMESHARE :: 1`
- `POLICY_RR :: 2`
- `POLICY_FIFO :: 4`
- `POLICYCLASS_FIXEDPRI :: POLICY_RR | POLICY_FIFO`
- `TASK_INFO_MAX :: 1024`
- `TASK_BASIC_INFO_32 :: 4`
- `TASK_BASIC2_INFO_32 :: 6`
- `TASK_BASIC_INFO_64 :: 5`
- `TASK_BASIC_INFO :: TASK_BASIC_INFO_64`
- `TASK_EVENTS_INFO :: 2`
- `TASK_THREAD_TIMES_INFO :: 3`
- `TASK_ABSOLUTETIME_INFO :: 1`
- `TASK_KERNELMEMORY_INFO :: 7`
- `TASK_SECURITY_TOKEN :: 13`
- `TASK_AUDIT_TOKEN :: 15`
- `TASK_AFFINITY_TAG_INFO :: 16`
- `TASK_DYLD_INFO :: 17`
- `TASK_DYLD_ALL_IMAGE_INFO_32 :: 0`
- `TASK_DYLD_ALL_IMAGE_INFO_64 :: 1`
- `TASK_EXTMOD_INFO :: 19`
- `MACH_TASK_BASIC_INFO :: 20`
- `TASK_POWER_INFO :: 21`
- `TASK_VM_INFO :: 22`
- `TASK_VM_INFO_PURGEABLE :: 23`
- `TASK_TRACE_MEMORY_INFO :: 24`
- `TASK_WAIT_STATE_INFO :: 25`
- `TASK_POWER_INFO_V2 :: 26`
- `TASK_VM_INFO_PURGEABLE_ACCOUNT :: 27`
- `TASK_FLAGS_INFO :: 28`
- `TF_LP64 :: 0x00000001`
- `TASK_DEBUG_INFO_INTERNAL :: 29`
- `TASK_SCHED_TIMESHARE_INFO :: 10`
- `TASK_SCHED_RR_INFO :: 11`
- `TASK_SCHED_FIFO_INFO :: 12`
- `TASK_SCHED_INFO :: 14`
- `TASK_CATEGORY_POLICY :: 1`
- `TASK_SUPPRESSION_POLICY :: 3`
- `TASK_POLICY_STATE :: 4`
- `TASK_BASE_QOS_POLICY :: 8`
- `TASK_OVERRIDE_QOS_POLICY :: 9`
- `TASK_BASE_LATENCY_QOS_POLICY :: 10`
- `TASK_BASE_THROUGHPUT_QOS_POLICY :: 11`
- `PROC_FLAG_DARWINBG :: 0x8000`
- `PROC_FLAG_EXT_DARWINBG :: 0x10000`
- `PROC_FLAG_IOS_APPLEDAEMON :: 0x20000`
- `PROC_FLAG_IOS_IMPPROMOTION :: 0x80000`
- `PROC_FLAG_ADAPTIVE :: 0x100000`
- `PROC_FLAG_ADAPTIVE_IMPORTANT :: 0x200000`
- `PROC_FLAG_IMPORTANCE_DONOR :: 0x400000`
- `PROC_FLAG_SUPPRESSED :: 0x800000`
- `PROC_FLAG_APPLICATION :: 0x1000000`
- `PROC_FLAG_IOS_APPLICATION :: PROC_FLAG_APPLICATION`
- `THREAD_STANDARD_POLICY :: 1`
- `THREAD_STANDARD_POLICY_COUNT :: 0`
- `THREAD_EXTENDED_POLICY :: 1`
- `THREAD_TIME_CONSTRAINT_POLICY :: 2`
- `THREAD_PRECEDENCE_POLICY :: 3`
- `THREAD_AFFINITY_POLICY :: 4`
- `THREAD_AFFINITY_TAG_NULL :: 0`
- `THREAD_BACKGROUND_POLICY :: 5`
- `THREAD_BACKGROUND_POLICY_DARWIN_BG :: 0x1000`
- `THREAD_LATENCY_QOS_POLICY :: 7`
- `THREAD_THROUGHPUT_QOS_POLICY :: 8`
- `SYSTEM_CLOCK :: 0`
- `CALENDAR_CLOCK :: 1`
- `REALTIME_CLOCK :: 0`
- `CLOCK_GET_TIME_RES :: 1`
- `CLOCK_ALARM_CURRES :: 3`
- `CLOCK_ALARM_MINRES :: 4`
- `CLOCK_ALARM_MAXRES :: 5`
- `NSEC_PER_USEC :: 1000`
- `USEC_PER_SEC :: 1000000`
- `NSEC_PER_SEC :: 1000000000`
- `NSEC_PER_MSEC :: 1000000`
- `ALRMTYPE :: 0xff`
- `TIME_ABSOLUTE :: 0x00`
- `TIME_RELATIVE :: 0x01`
- `TASK_NULL :: cast(task_t) 0`
- `TASK_NAME_NULL :: cast(task_name_t) 0`
- `TASK_INSPECT_NULL :: cast(task_inspect_t) 0`
- `THREAD_NULL :: cast(thread_t) 0`
- `THREAD_INSPECT_NULL :: cast(thread_inspect_t) 0`
- `TID_NULL :: cast(u64) 0`
- `THR_ACT_NULL :: cast(thread_act_t) 0`
- `IPC_SPACE_NULL :: cast(ipc_space_t) 0`
- `IPC_SPACE_INSPECT_NULL :: cast(ipc_space_inspect_t) 0`
- `COALITION_NULL :: cast(coalition_t) 0`
- `HOST_NULL :: cast(host_t) 0`
- `HOST_PRIV_NULL :: cast(host_priv_t) 0`
- `HOST_SECURITY_NULL :: cast(host_security_t) 0`
- `PROCESSOR_SET_NULL :: cast(processor_set_t) 0`
- `PROCESSOR_NULL :: cast(processor_t) 0`
- `SEMAPHORE_NULL :: cast(semaphore_t) 0`
- `LOCK_SET_NULL :: cast(lock_set_t) 0`
- `LEDGER_NULL :: cast(ledger_t) 0`
- `ALARM_NULL :: cast(alarm_t) 0`
- `UND_SERVER_NULL :: cast(UNDServerRef) 0`
- `LEDGER_ITEM_INFINITY :: cast,trunc(ledger_item_t) (~0)`
- `LEDGER_LIMIT_INFINITY :: cast(ledger_amount_t) ((1 << 63) - 1)`
- `ZONE_NAME_MAX_LEN :: 80`
- `MACH_ZONE_NAME_MAX_LEN :: 80`
- `MACH_MEMORY_INFO_NAME_MAX_LEN :: 80`
- `SYNC_POLICY_FIFO :: 0x0`
- `SYNC_POLICY_FIXED_PRIORITY :: 0x1`
- `SYNC_POLICY_REVERSED :: 0x2`
- `SYNC_POLICY_ORDER_MASK :: 0x3`
- `SYNC_POLICY_LIFO :: SYNC_POLICY_FIFO|SYNC_POLICY_REVERSED`
- `SYNC_POLICY_MAX :: 0x7`
- `task_MSG_COUNT :: 52`
- `NDR_record_t :: struct`
- `vm_offset_t :: u64`
- `vm_size_t :: u64`
- `mach_vm_address_t :: u64`
- `mach_vm_offset_t :: u64`
- `mach_vm_size_t :: u64`
- `vm_map_offset_t :: u64`
- `vm_map_address_t :: u64`
- `vm_map_size_t :: u64`
- `mach_port_context_t :: mach_vm_address_t`
- `mach_port_name_t :: u32`
- `mach_port_name_array_t :: *mach_port_name_t`
- `mach_port_array_t :: *mach_port_t`
- `mach_port_right_t :: u32`
- `mach_port_type_t :: u32`
- `mach_port_type_array_t :: *mach_port_type_t`
- `mach_port_urefs_t :: u32`
- `mach_port_delta_t :: s32`
- `mach_port_seqno_t :: u32`
- `mach_port_mscount_t :: u32`
- `mach_port_msgcount_t :: u32`
- `mach_port_rights_t :: u32`
- `mach_port_srights_t :: u32`
- `mach_port_status :: struct`
- `mach_port_status_t :: mach_port_status`
- `mach_port_limits :: struct`
- `mach_port_limits_t :: mach_port_limits`
- `mach_port_info_ext :: struct`
- `mach_port_info_ext_t :: mach_port_info_ext`
- `mach_port_info_t :: *s32`
- `mach_port_flavor_t :: s32`
- `mach_port_qos :: struct`
- `set_name :: (fields: *mach_port_qos_t, value: bool)`
- `get_name :: (fields: mach_port_qos_t) -> bool`
- `set_prealloc :: (fields: *mach_port_qos_t, value: bool)`
- `get_prealloc :: (fields: mach_port_qos_t) -> bool`
- `set_pad1 :: (fields: *mach_port_qos_t, value: u32)`
- `get_pad1 :: (fields: mach_port_qos_t) -> u32`
- `mach_port_qos_t :: mach_port_qos`
- `mach_port_options :: struct`
- `mach_port_options_t :: mach_port_options`
- `mach_port_options_ptr_t :: *mach_port_options_t`
- `mach_port_guard_exception_codes :: enum_flags u32`
- `mach_msg_timeout_t :: u32`
- `mach_msg_bits_t :: u32`
- `mach_msg_size_t :: u32`
- `mach_msg_id_t :: s32`
- `mach_msg_priority_t :: u32`
- `mach_msg_type_name_t :: u32`
- `mach_msg_copy_options_t :: u32`
- `mach_msg_descriptor_type_t :: u32`
- `mach_msg_type_descriptor_t :: struct`
- `set_pad3 :: (fields: *mach_msg_type_descriptor_t, value: u32)`
- `get_pad3 :: (fields: mach_msg_type_descriptor_t) -> u32`
- `set_type :: (fields: *mach_msg_type_descriptor_t, value: mach_msg_descriptor_type_t)`
- `get_type :: (fields: mach_msg_type_descriptor_t) -> mach_msg_descriptor_type_t`
- `mach_msg_port_descriptor_t :: struct`
- `set_pad2 :: (fields: *mach_msg_port_descriptor_t, value: u32)`
- `get_pad2 :: (fields: mach_msg_port_descriptor_t) -> u32`
- `set_disposition :: (fields: *mach_msg_port_descriptor_t, value: mach_msg_type_name_t)`
- `get_disposition :: (fields: mach_msg_port_descriptor_t) -> mach_msg_type_name_t`
- `set_type :: (fields: *mach_msg_port_descriptor_t, value: mach_msg_descriptor_type_t)`
- `get_type :: (fields: mach_msg_port_descriptor_t) -> mach_msg_descriptor_type_t`
- `mach_msg_ool_descriptor32_t :: struct`
- `set_deallocate :: (fields: *mach_msg_ool_descriptor32_t, value: u32)`
- `get_deallocate :: (fields: mach_msg_ool_descriptor32_t) -> u32`
- `set_copy :: (fields: *mach_msg_ool_descriptor32_t, value: mach_msg_copy_options_t)`
- `get_copy :: (fields: mach_msg_ool_descriptor32_t) -> mach_msg_copy_options_t`
- `set_pad1 :: (fields: *mach_msg_ool_descriptor32_t, value: u32)`
- `get_pad1 :: (fields: mach_msg_ool_descriptor32_t) -> u32`
- `set_type :: (fields: *mach_msg_ool_descriptor32_t, value: mach_msg_descriptor_type_t)`
- `get_type :: (fields: mach_msg_ool_descriptor32_t) -> mach_msg_descriptor_type_t`
- `mach_msg_ool_descriptor64_t :: struct`
- `set_deallocate :: (fields: *mach_msg_ool_descriptor64_t, value: u32)`
- `get_deallocate :: (fields: mach_msg_ool_descriptor64_t) -> u32`
- `set_copy :: (fields: *mach_msg_ool_descriptor64_t, value: mach_msg_copy_options_t)`
- `get_copy :: (fields: mach_msg_ool_descriptor64_t) -> mach_msg_copy_options_t`
- `set_pad1 :: (fields: *mach_msg_ool_descriptor64_t, value: u32)`
- `get_pad1 :: (fields: mach_msg_ool_descriptor64_t) -> u32`
- `set_type :: (fields: *mach_msg_ool_descriptor64_t, value: mach_msg_descriptor_type_t)`
- `get_type :: (fields: mach_msg_ool_descriptor64_t) -> mach_msg_descriptor_type_t`
- `mach_msg_ool_descriptor_t :: struct`
- `set_deallocate :: (fields: *mach_msg_ool_descriptor_t, value: u32)`
- `get_deallocate :: (fields: mach_msg_ool_descriptor_t) -> u32`
- `set_copy :: (fields: *mach_msg_ool_descriptor_t, value: mach_msg_copy_options_t)`
- `get_copy :: (fields: mach_msg_ool_descriptor_t) -> mach_msg_copy_options_t`
- `set_pad1 :: (fields: *mach_msg_ool_descriptor_t, value: u32)`
- `get_pad1 :: (fields: mach_msg_ool_descriptor_t) -> u32`
- `set_type :: (fields: *mach_msg_ool_descriptor_t, value: mach_msg_descriptor_type_t)`
- `get_type :: (fields: mach_msg_ool_descriptor_t) -> mach_msg_descriptor_type_t`
- `mach_msg_ool_ports_descriptor32_t :: struct`
- `set_deallocate :: (fields: *mach_msg_ool_ports_descriptor32_t, value: u32)`
- `get_deallocate :: (fields: mach_msg_ool_ports_descriptor32_t) -> u32`
- `set_copy :: (fields: *mach_msg_ool_ports_descriptor32_t, value: mach_msg_copy_options_t)`
- `get_copy :: (fields: mach_msg_ool_ports_descriptor32_t) -> mach_msg_copy_options_t`
- `set_disposition :: (fields: *mach_msg_ool_ports_descriptor32_t, value: mach_msg_type_name_t)`
- `get_disposition :: (fields: mach_msg_ool_ports_descriptor32_t) -> mach_msg_type_name_t`
- `set_type :: (fields: *mach_msg_ool_ports_descriptor32_t, value: mach_msg_descriptor_type_t)`
- `get_type :: (fields: mach_msg_ool_ports_descriptor32_t) -> mach_msg_descriptor_type_t`
- `mach_msg_ool_ports_descriptor64_t :: struct`
- `set_deallocate :: (fields: *mach_msg_ool_ports_descriptor64_t, value: u32)`
- `get_deallocate :: (fields: mach_msg_ool_ports_descriptor64_t) -> u32`
- `set_copy :: (fields: *mach_msg_ool_ports_descriptor64_t, value: mach_msg_copy_options_t)`
- `get_copy :: (fields: mach_msg_ool_ports_descriptor64_t) -> mach_msg_copy_options_t`
- `set_disposition :: (fields: *mach_msg_ool_ports_descriptor64_t, value: mach_msg_type_name_t)`
- `get_disposition :: (fields: mach_msg_ool_ports_descriptor64_t) -> mach_msg_type_name_t`
- `set_type :: (fields: *mach_msg_ool_ports_descriptor64_t, value: mach_msg_descriptor_type_t)`
- `get_type :: (fields: mach_msg_ool_ports_descriptor64_t) -> mach_msg_descriptor_type_t`
- `mach_msg_ool_ports_descriptor_t :: struct`
- `set_deallocate :: (fields: *mach_msg_ool_ports_descriptor_t, value: u32)`
- `get_deallocate :: (fields: mach_msg_ool_ports_descriptor_t) -> u32`
- `set_copy :: (fields: *mach_msg_ool_ports_descriptor_t, value: mach_msg_copy_options_t)`
- `get_copy :: (fields: mach_msg_ool_ports_descriptor_t) -> mach_msg_copy_options_t`
- `set_disposition :: (fields: *mach_msg_ool_ports_descriptor_t, value: mach_msg_type_name_t)`
- `get_disposition :: (fields: mach_msg_ool_ports_descriptor_t) -> mach_msg_type_name_t`
- `set_type :: (fields: *mach_msg_ool_ports_descriptor_t, value: mach_msg_descriptor_type_t)`
- `get_type :: (fields: mach_msg_ool_ports_descriptor_t) -> mach_msg_descriptor_type_t`
- `mach_msg_descriptor_t :: struct`
- `mach_msg_body_t :: struct`
- `mach_msg_header_t :: struct`
- `mach_msg_base_t :: struct`
- `mach_msg_trailer_type_t :: u32`
- `mach_msg_trailer_size_t :: u32`
- `mach_msg_trailer_info_t :: *u8`
- `mach_msg_trailer_t :: struct`
- `mach_msg_seqno_trailer_t :: struct`
- `security_token_t :: struct`
- `mach_msg_security_trailer_t :: struct`
- `audit_token_t :: struct`
- `mach_msg_audit_trailer_t :: struct`
- `mach_msg_context_trailer_t :: struct`
- `msg_labels_t :: struct`
- `mach_msg_mac_trailer_t :: struct`
- `mach_msg_max_trailer_t :: mach_msg_mac_trailer_t`
- `mach_msg_format_0_trailer_t :: mach_msg_security_trailer_t`
- `mach_msg_options_t :: s32`
- `mach_msg_empty_send_t :: struct`
- `mach_msg_empty_rcv_t :: struct`
- `mach_msg_empty_t :: struct`
- `mach_msg_type_size_t :: u32`
- `mach_msg_type_number_t :: u32`
- `mach_msg_option_t :: s32`
- `mach_msg_return_t :: kern_return_t`
- `mach_msg_overwrite :: (msg: *mach_msg_header_t, option: mach_msg_option_t, send_size: mach_msg_size_t, rcv_size: mach_msg_size_t, rcv_name: mach_port_name_t, timeout: mach_msg_timeout_t, notify: mach_port_name_t, rcv_msg: *mach_msg_header_t, rcv_limit: mach_msg_size_t) -> mach_msg_return_t #foreign libc`
- `mach_msg :: (msg: *mach_msg_header_t, option: mach_msg_option_t, send_size: mach_msg_size_t, rcv_size: mach_msg_size_t, rcv_name: mach_port_name_t, timeout: mach_msg_timeout_t, notify: mach_port_name_t) -> mach_msg_return_t #foreign libc`
- `mach_voucher_deallocate :: (voucher: mach_port_name_t) -> kern_return_t #foreign libc`
- `vm_statistics :: struct`
- `vm_statistics_t :: *vm_statistics`
- `vm_statistics_data_t :: vm_statistics`
- `vm_statistics64 :: struct`
- `vm_statistics64_t :: *vm_statistics64`
- `vm_statistics64_data_t :: vm_statistics64`
- `vm_extmod_statistics :: struct`
- `vm_extmod_statistics_t :: *vm_extmod_statistics`
- `vm_extmod_statistics_data_t :: vm_extmod_statistics`
- `vm_purgeable_stat :: struct`
- `vm_purgeable_stat_t :: vm_purgeable_stat`
- `vm_purgeable_info :: struct`
- `vm_purgeable_info_t :: *vm_purgeable_info`
- `cpu_type_t :: s32`
- `cpu_subtype_t :: s32`
- `cpu_threadtype_t :: s32`
- `time_value :: struct`
- `time_value_t :: time_value`
- `pointer_t :: vm_offset_t`
- `vm_address_t :: vm_offset_t`
- `addr64_t :: u64`
- `reg64_t :: u32`
- `ppnum_t :: u32`
- `vm_map_t :: mach_port_t`
- `vm_object_offset_t :: u64`
- `vm_object_size_t :: u64`
- `upl_t :: mach_port_t`
- `vm_named_entry_t :: mach_port_t`
- `thread_state_t :: *u32`
- `thread_state_data_t :: [614] u32`
- `thread_state_flavor_t :: s32`
- `thread_state_flavor_array_t :: *thread_state_flavor_t`
- `exception_type_t :: s32`
- `exception_data_type_t :: s32`
- `mach_exception_data_type_t :: s64`
- `exception_behavior_t :: s32`
- `exception_data_t :: *exception_data_type_t`
- `mach_exception_data_t :: *mach_exception_data_type_t`
- `exception_mask_t :: u32`
- `exception_mask_array_t :: *exception_mask_t`
- `exception_behavior_array_t :: *exception_behavior_t`
- `exception_flavor_array_t :: *thread_state_flavor_t`
- `exception_port_array_t :: *mach_port_t`
- `mach_exception_code_t :: mach_exception_data_type_t`
- `mach_exception_subcode_t :: mach_exception_data_type_t`
- `mach_voucher_t :: mach_port_t`
- `mach_voucher_name_t :: mach_port_name_t`
- `mach_voucher_name_array_t :: *mach_voucher_name_t`
- `ipc_voucher_t :: mach_voucher_t`
- `mach_voucher_selector_t :: u32`
- `mach_voucher_attr_key_t :: u32`
- `mach_voucher_attr_key_array_t :: *mach_voucher_attr_key_t`
- `mach_voucher_attr_content_t :: *u8`
- `mach_voucher_attr_content_size_t :: u32`
- `mach_voucher_attr_command_t :: u32`
- `mach_voucher_attr_recipe_command_t :: u32`
- `mach_voucher_attr_recipe_command_array_t :: *mach_voucher_attr_recipe_command_t`
- `mach_voucher_attr_recipe_data :: struct`
- `mach_voucher_attr_recipe_data_t :: mach_voucher_attr_recipe_data`
- `mach_voucher_attr_recipe_t :: *mach_voucher_attr_recipe_data_t`
- `mach_voucher_attr_recipe_size_t :: mach_msg_type_number_t`
- `mach_voucher_attr_raw_recipe_t :: *u8`
- `mach_voucher_attr_raw_recipe_array_t :: mach_voucher_attr_raw_recipe_t`
- `mach_voucher_attr_raw_recipe_size_t :: mach_msg_type_number_t`
- `mach_voucher_attr_raw_recipe_array_size_t :: mach_msg_type_number_t`
- `mach_voucher_attr_manager_t :: mach_port_t`
- `mach_voucher_attr_control_t :: mach_port_t`
- `ipc_voucher_attr_manager_t :: mach_port_t`
- `ipc_voucher_attr_control_t :: mach_port_t`
- `mach_voucher_attr_value_handle_t :: u64`
- `mach_voucher_attr_value_handle_array_t :: *mach_voucher_attr_value_handle_t`
- `mach_voucher_attr_value_handle_array_size_t :: mach_msg_type_number_t`
- `mach_voucher_attr_value_reference_t :: u32`
- `mach_voucher_attr_value_flags_t :: u32`
- `mach_voucher_attr_control_flags_t :: u32`
- `mach_voucher_attr_importance_refs :: u32`
- `policy_t :: s32`
- `policy_info_t :: *s32`
- `policy_base_t :: *s32`
- `policy_limit_t :: *s32`
- `policy_timeshare_base :: struct`
- `policy_timeshare_limit :: struct`
- `policy_timeshare_info :: struct`
- `policy_timeshare_base_t :: *policy_timeshare_base`
- `policy_timeshare_limit_t :: *policy_timeshare_limit`
- `policy_timeshare_info_t :: *policy_timeshare_info`
- `policy_timeshare_base_data_t :: policy_timeshare_base`
- `policy_timeshare_limit_data_t :: policy_timeshare_limit`
- `policy_timeshare_info_data_t :: policy_timeshare_info`
- `policy_rr_base :: struct`
- `policy_rr_limit :: struct`
- `policy_rr_info :: struct`
- `policy_rr_base_t :: *policy_rr_base`
- `policy_rr_limit_t :: *policy_rr_limit`
- `policy_rr_info_t :: *policy_rr_info`
- `policy_rr_base_data_t :: policy_rr_base`
- `policy_rr_limit_data_t :: policy_rr_limit`
- `policy_rr_info_data_t :: policy_rr_info`
- `policy_fifo_base :: struct`
- `policy_fifo_limit :: struct`
- `policy_fifo_info :: struct`
- `policy_fifo_base_t :: *policy_fifo_base`
- `policy_fifo_limit_t :: *policy_fifo_limit`
- `policy_fifo_info_t :: *policy_fifo_info`
- `policy_fifo_base_data_t :: policy_fifo_base`
- `policy_fifo_limit_data_t :: policy_fifo_limit`
- `policy_fifo_info_data_t :: policy_fifo_info`
- `policy_bases :: struct`
- `policy_limits :: struct`
- `policy_infos :: struct`
- `policy_base_data_t :: policy_bases`
- `policy_limit_data_t :: policy_limits`
- `policy_info_data_t :: policy_infos`
- `task_flavor_t :: u32`
- `task_info_t :: *s32`
- `task_info_data_t :: [1024] s32`
- `task_basic_info_32 :: struct`
- `task_basic_info_32_data_t :: task_basic_info_32`
- `task_basic_info_32_t :: *task_basic_info_32`
- `task_basic_info_64 :: struct`
- `task_basic_info_64_data_t :: task_basic_info_64`
- `task_basic_info_64_t :: *task_basic_info_64`
- `task_basic_info :: struct`
- `task_basic_info_data_t :: task_basic_info`
- `task_basic_info_t :: *task_basic_info`
- `task_events_info :: struct`
- `task_events_info_data_t :: task_events_info`
- `task_events_info_t :: *task_events_info`
- `task_thread_times_info :: struct`
- `task_thread_times_info_data_t :: task_thread_times_info`
- `task_thread_times_info_t :: *task_thread_times_info`
- `task_absolutetime_info :: struct`
- `task_absolutetime_info_data_t :: task_absolutetime_info`
- `task_absolutetime_info_t :: *task_absolutetime_info`
- `task_kernelmemory_info :: struct`
- `task_kernelmemory_info_data_t :: task_kernelmemory_info`
- `task_kernelmemory_info_t :: *task_kernelmemory_info`
- `task_affinity_tag_info :: struct`
- `task_affinity_tag_info_data_t :: task_affinity_tag_info`
- `task_affinity_tag_info_t :: *task_affinity_tag_info`
- `task_dyld_info :: struct`
- `task_dyld_info_data_t :: task_dyld_info`
- `task_dyld_info_t :: *task_dyld_info`
- `task_extmod_info :: struct`
- `task_extmod_info_data_t :: task_extmod_info`
- `task_extmod_info_t :: *task_extmod_info`
- `mach_task_basic_info :: struct`
- `mach_task_basic_info_data_t :: mach_task_basic_info`
- `mach_task_basic_info_t :: *mach_task_basic_info`
- `task_power_info :: struct`
- `task_power_info_data_t :: task_power_info`
- `task_power_info_t :: *task_power_info`
- `task_vm_info :: struct`
- `task_vm_info_data_t :: task_vm_info`
- `task_vm_info_t :: *task_vm_info`
- `task_purgable_info_t :: vm_purgeable_info`
- `task_trace_memory_info :: struct`
- `task_trace_memory_info_data_t :: task_trace_memory_info`
- `task_trace_memory_info_t :: *task_trace_memory_info`
- `task_wait_state_info :: struct`
- `task_wait_state_info_data_t :: task_wait_state_info`
- `task_wait_state_info_t :: *task_wait_state_info`
- `gpu_energy_data :: struct`
- `gpu_energy_data_t :: *gpu_energy_data`
- `task_power_info_v2 :: struct`
- `task_power_info_v2_data_t :: task_power_info_v2`
- `task_power_info_v2_t :: *task_power_info_v2`
- `task_flags_info :: struct`
- `task_flags_info_data_t :: task_flags_info`
- `task_flags_info_t :: *task_flags_info`
- `task_inspect_flavor_t :: u32`
- `task_inspect_flavor :: enum u32`
- `task_inspect_basic_counts :: struct`
- `task_inspect_basic_counts_data_t :: task_inspect_basic_counts`
- `task_inspect_basic_counts_t :: *task_inspect_basic_counts`
- `task_inspect_info_t :: *s32`
- `task_policy_flavor_t :: u32`
- `task_policy_t :: *s32`
- `task_role :: enum s32`
- `task_role_t :: s32`
- `task_category_policy :: struct`
- `task_category_policy_data_t :: task_category_policy`
- `task_category_policy_t :: *task_category_policy`
- `task_latency_qos :: enum u32`
- `task_latency_qos_t :: s32`
- `task_throughput_qos :: enum u32`
- `task_throughput_qos_t :: s32`
- `task_qos_policy :: struct`
- `task_qos_policy_t :: *task_qos_policy`
- `thread_policy_flavor_t :: u32`
- `thread_policy_t :: *s32`
- `thread_standard_policy :: struct`
- `thread_standard_policy_data_t :: thread_standard_policy`
- `thread_standard_policy_t :: *thread_standard_policy`
- `thread_extended_policy :: struct`
- `thread_extended_policy_data_t :: thread_extended_policy`
- `thread_extended_policy_t :: *thread_extended_policy`
- `thread_time_constraint_policy :: struct`
- `thread_time_constraint_policy_data_t :: thread_time_constraint_policy`
- `thread_time_constraint_policy_t :: *thread_time_constraint_policy`
- `thread_precedence_policy :: struct`
- `thread_precedence_policy_data_t :: thread_precedence_policy`
- `thread_precedence_policy_t :: *thread_precedence_policy`
- `thread_affinity_policy :: struct`
- `thread_affinity_policy_data_t :: thread_affinity_policy`
- `thread_affinity_policy_t :: *thread_affinity_policy`
- `thread_background_policy :: struct`
- `thread_background_policy_data_t :: thread_background_policy`
- `thread_background_policy_t :: *thread_background_policy`
- `thread_latency_qos_t :: s32`
- `thread_latency_qos_policy :: struct`
- `thread_latency_qos_policy_data_t :: thread_latency_qos_policy`
- `thread_latency_qos_policy_t :: *thread_latency_qos_policy`
- `thread_throughput_qos_t :: s32`
- `thread_throughput_qos_policy :: struct`
- `thread_throughput_qos_policy_data_t :: thread_throughput_qos_policy`
- `thread_throughput_qos_policy_t :: *thread_throughput_qos_policy`
- `alarm_type_t :: s32`
- `sleep_type_t :: s32`
- `clock_id_t :: s32`
- `clock_flavor_t :: s32`
- `clock_attr_t :: *s32`
- `clock_res_t :: s32`
- `mach_timespec :: struct`
- `mach_timespec_t :: mach_timespec`
- `fsid :: struct`
- `fsid_t :: fsid`
- `fsobj_id :: struct`
- `fsobj_id_t :: fsobj_id`
- `dyld_kernel_image_info :: struct`
- `dyld_kernel_process_info :: struct`
- `dyld_kernel_image_info_t :: dyld_kernel_image_info`
- `dyld_kernel_process_info_t :: dyld_kernel_process_info`
- `dyld_kernel_image_info_array_t :: *dyld_kernel_image_info_t`
- `task_t :: mach_port_t`
- `task_name_t :: mach_port_t`
- `task_inspect_t :: mach_port_t`
- `task_suspension_token_t :: mach_port_t`
- `thread_t :: mach_port_t`
- `thread_act_t :: mach_port_t`
- `thread_inspect_t :: mach_port_t`
- `ipc_space_t :: mach_port_t`
- `ipc_space_inspect_t :: mach_port_t`
- `coalition_t :: mach_port_t`
- `host_t :: mach_port_t`
- `host_priv_t :: mach_port_t`
- `host_security_t :: mach_port_t`
- `processor_t :: mach_port_t`
- `processor_set_t :: mach_port_t`
- `processor_set_control_t :: mach_port_t`
- `semaphore_t :: mach_port_t`
- `lock_set_t :: mach_port_t`
- `ledger_t :: mach_port_t`
- `alarm_t :: mach_port_t`
- `clock_serv_t :: mach_port_t`
- `clock_ctrl_t :: mach_port_t`
- `processor_set_name_t :: processor_set_t`
- `clock_reply_t :: mach_port_t`
- `bootstrap_t :: mach_port_t`
- `mem_entry_name_port_t :: mach_port_t`
- `exception_handler_t :: mach_port_t`
- `exception_handler_array_t :: *exception_handler_t`
- `vm_task_entry_t :: mach_port_t`
- `io_master_t :: mach_port_t`
- `UNDServerRef :: mach_port_t`
- `task_array_t :: *task_t`
- `thread_array_t :: *thread_t`
- `processor_set_array_t :: *processor_set_t`
- `processor_set_name_array_t :: *processor_set_t`
- `processor_array_t :: *processor_t`
- `thread_act_array_t :: *thread_act_t`
- `ledger_array_t :: *ledger_t`
- `task_port_t :: task_t`
- `task_port_array_t :: task_array_t`
- `thread_port_t :: thread_t`
- `thread_port_array_t :: thread_array_t`
- `ipc_space_port_t :: ipc_space_t`
- `host_name_t :: host_t`
- `host_name_port_t :: host_t`
- `processor_set_port_t :: processor_set_t`
- `processor_set_name_port_t :: processor_set_t`
- `processor_set_name_port_array_t :: processor_set_array_t`
- `processor_set_control_port_t :: processor_set_t`
- `processor_port_t :: processor_t`
- `processor_port_array_t :: processor_array_t`
- `thread_act_port_t :: thread_act_t`
- `thread_act_port_array_t :: thread_act_array_t`
- `semaphore_port_t :: semaphore_t`
- `lock_set_port_t :: lock_set_t`
- `ledger_port_t :: ledger_t`
- `ledger_port_array_t :: ledger_array_t`
- `alarm_port_t :: alarm_t`
- `clock_serv_port_t :: clock_serv_t`
- `clock_ctrl_port_t :: clock_ctrl_t`
- `exception_port_t :: exception_handler_t`
- `exception_port_arrary_t :: exception_handler_array_t`
- `ledger_item_t :: u32`
- `ledger_amount_t :: s64`
- `emulation_vector_t :: *mach_vm_offset_t`
- `user_subsystem_t :: *u8`
- `labelstr_t :: *u8`
- `ipc_info_space :: struct`
- `ipc_info_space_t :: ipc_info_space`
- `ipc_info_space_basic :: struct`
- `ipc_info_space_basic_t :: ipc_info_space_basic`
- `ipc_info_name :: struct`
- `ipc_info_name_t :: ipc_info_name`
- `ipc_info_name_array_t :: *ipc_info_name_t`
- `ipc_info_tree_name :: struct`
- `ipc_info_tree_name_t :: ipc_info_tree_name`
- `ipc_info_tree_name_array_t :: *ipc_info_tree_name_t`
- `zone_name :: struct`
- `zone_name_t :: zone_name`
- `zone_name_array_t :: *zone_name_t`
- `zone_info :: struct`
- `zone_info_t :: zone_info`
- `zone_info_array_t :: *zone_info_t`
- `mach_zone_name :: struct`
- `mach_zone_name_t :: mach_zone_name`
- `mach_zone_name_array_t :: *mach_zone_name_t`
- `mach_zone_info_data :: struct`
- `mach_zone_info_t :: mach_zone_info_data`
- `mach_zone_info_array_t :: *mach_zone_info_t`
- `task_zone_info_data :: struct`
- `task_zone_info_t :: task_zone_info_data`
- `task_zone_info_array_t :: *task_zone_info_t`
- `mach_memory_info :: struct`
- `mach_memory_info_t :: mach_memory_info`
- `mach_memory_info_array_t :: *mach_memory_info_t`
- `sync_policy_t :: s32`
- `semaphore_signal :: (semaphore: semaphore_t) -> kern_return_t #foreign libc`
- `semaphore_signal_all :: (semaphore: semaphore_t) -> kern_return_t #foreign libc`
- `semaphore_wait :: (semaphore: semaphore_t) -> kern_return_t #foreign libc`
- `semaphore_timedwait :: (semaphore: semaphore_t, wait_time: mach_timespec_t) -> kern_return_t #foreign libc`
- `semaphore_timedwait_signal :: (wait_semaphore: semaphore_t, signal_semaphore: semaphore_t, wait_time: mach_timespec_t) -> kern_return_t #foreign libc`
- `semaphore_wait_signal :: (wait_semaphore: semaphore_t, signal_semaphore: semaphore_t) -> kern_return_t #foreign libc`
- `semaphore_signal_thread :: (semaphore: semaphore_t, thread: thread_t) -> kern_return_t #foreign libc`
- `task_create :: (target_task: task_t, ledgers: ledger_port_array_t, ledgersCnt: mach_msg_type_number_t, inherit_memory: u32, child_task: *task_t) -> kern_return_t #foreign libc`
- `task_terminate :: (target_task: task_t) -> kern_return_t #foreign libc`
- `task_threads :: (target_task: task_inspect_t, act_list: *thread_act_port_array_t, act_listCnt: *mach_msg_type_number_t) -> kern_return_t #foreign libc`
- `mach_ports_register :: (target_task: task_t, init_port_set: mach_port_array_t, init_port_setCnt: mach_msg_type_number_t) -> kern_return_t #foreign libc`
- `mach_ports_lookup :: (target_task: task_t, init_port_set: *mach_port_array_t, init_port_setCnt: *mach_msg_type_number_t) -> kern_return_t #foreign libc`
- `task_info :: (target_task: task_name_t, flavor: task_flavor_t, task_info_out: task_info_t, task_info_outCnt: *mach_msg_type_number_t) -> kern_return_t #foreign libc`
- `task_set_info :: (target_task: task_t, flavor: task_flavor_t, task_info_in: task_info_t, task_info_inCnt: mach_msg_type_number_t) -> kern_return_t #foreign libc`
- `task_suspend :: (target_task: task_t) -> kern_return_t #foreign libc`
- `task_resume :: (target_task: task_t) -> kern_return_t #foreign libc`
- `task_get_special_port :: (task: task_inspect_t, which_port: s32, special_port: *mach_port_t) -> kern_return_t #foreign libc`
- `task_set_special_port :: (task: task_t, which_port: s32, special_port: mach_port_t) -> kern_return_t #foreign libc`
- `thread_create :: (parent_task: task_t, child_act: *thread_act_t) -> kern_return_t #foreign libc`
- `thread_create_running :: (parent_task: task_t, flavor: thread_state_flavor_t, new_state: thread_state_t, new_stateCnt: mach_msg_type_number_t, child_act: *thread_act_t) -> kern_return_t #foreign libc`
- `task_set_exception_ports :: (task: task_t, exception_mask: exception_mask_t, new_port: mach_port_t, behavior: exception_behavior_t, new_flavor: thread_state_flavor_t) -> kern_return_t #foreign libc`
- `task_get_exception_ports :: (task: task_inspect_t, exception_mask: exception_mask_t, masks: exception_mask_array_t, masksCnt: *mach_msg_type_number_t, old_handlers: exception_port_arrary_t, old_behaviors: exception_behavior_array_t, old_flavors: exception_flavor_array_t) -> kern_return_t #foreign libc`
- `task_swap_exception_ports :: (task: task_t, exception_mask: exception_mask_t, new_port: mach_port_t, behavior: exception_behavior_t, new_flavor: thread_state_flavor_t, masks: exception_mask_array_t, masksCnt: *mach_msg_type_number_t, old_handlerss: exception_port_arrary_t, old_behaviors: exception_behavior_array_t, old_flavors: exception_flavor_array_t) -> kern_return_t #foreign libc`
- `lock_set_create :: (task: task_t, new_lock_set: *lock_set_t, n_ulocks: s32, policy: s32) -> kern_return_t #foreign libc`
- `lock_set_destroy :: (task: task_t, lock_set: lock_set_t) -> kern_return_t #foreign libc`
- `semaphore_create :: (task: task_t, semaphore: *semaphore_t, policy: s32, value: s32) -> kern_return_t #foreign libc`
- `semaphore_destroy :: (task: task_t, semaphore: semaphore_t) -> kern_return_t #foreign libc`
- `task_policy_set :: (task: task_t, flavor: task_policy_flavor_t, policy_info: task_policy_t, policy_infoCnt: mach_msg_type_number_t) -> kern_return_t #foreign libc`
- `task_policy_get :: (task: task_t, flavor: task_policy_flavor_t, policy_info: task_policy_t, policy_infoCnt: *mach_msg_type_number_t, get_default: *u32) -> kern_return_t #foreign libc`
- `task_sample :: (task: task_t, reply: mach_port_t) -> kern_return_t #foreign libc`
- `task_policy :: (task: task_t, policy: policy_t, base: policy_base_t, baseCnt: mach_msg_type_number_t, set_limit: u32, change: u32) -> kern_return_t #foreign libc`
- `task_set_emulation :: (target_port: task_t, routine_entry_pt: vm_address_t, routine_number: s32) -> kern_return_t #foreign libc`
- `task_get_emulation_vector :: (task: task_t, vector_start: *s32, emulation_vector: *emulation_vector_t, emulation_vectorCnt: *mach_msg_type_number_t) -> kern_return_t #foreign libc`
- `task_set_emulation_vector :: (task: task_t, vector_start: s32, emulation_vector: emulation_vector_t, emulation_vectorCnt: mach_msg_type_number_t) -> kern_return_t #foreign libc`
- `task_set_ras_pc :: (target_task: task_t, basepc: vm_address_t, boundspc: vm_address_t) -> kern_return_t #foreign libc`
- `task_zone_info :: (target_task: task_t, names: *mach_zone_name_array_t, namesCnt: *mach_msg_type_number_t, info: *task_zone_info_array_t, infoCnt: *mach_msg_type_number_t) -> kern_return_t #foreign libc`
- `task_assign :: (task: task_t, new_set: processor_set_t, assign_threads: u32) -> kern_return_t #foreign libc`
- `task_assign_default :: (task: task_t, assign_threads: u32) -> kern_return_t #foreign libc`
- `task_get_assignment :: (task: task_t, assigned_set: *processor_set_name_t) -> kern_return_t #foreign libc`
- `task_set_policy :: (task: task_t, pset: processor_set_t, policy: policy_t, base: policy_base_t, baseCnt: mach_msg_type_number_t, limit: policy_limit_t, limitCnt: mach_msg_type_number_t, change: u32) -> kern_return_t #foreign libc`
- `task_get_state :: (task: task_t, flavor: thread_state_flavor_t, old_state: thread_state_t, old_stateCnt: *mach_msg_type_number_t) -> kern_return_t #foreign libc`
- `task_set_state :: (task: task_t, flavor: thread_state_flavor_t, new_state: thread_state_t, new_stateCnt: mach_msg_type_number_t) -> kern_return_t #foreign libc`
- `task_set_phys_footprint_limit :: (task: task_t, new_limit: s32, old_limit: *s32) -> kern_return_t #foreign libc`
- `task_suspend2 :: (target_task: task_t, suspend_token: *task_suspension_token_t) -> kern_return_t #foreign libc`
- `task_resume2 :: (suspend_token: task_suspension_token_t) -> kern_return_t #foreign libc`
- `task_purgable_info :: (task: task_t, stats: *task_purgable_info_t) -> kern_return_t #foreign libc`
- `task_get_mach_voucher :: (task: task_t, which: mach_voucher_selector_t, voucher: *ipc_voucher_t) -> kern_return_t #foreign libc`
- `task_set_mach_voucher :: (task: task_t, voucher: ipc_voucher_t) -> kern_return_t #foreign libc`
- `task_swap_mach_voucher :: (task: task_t, new_voucher: ipc_voucher_t, old_voucher: *ipc_voucher_t) -> kern_return_t #foreign libc`
- `task_generate_corpse :: (task: task_t, corpse_task_port: *mach_port_t) -> kern_return_t #foreign libc`
- `task_map_corpse_info :: (task: task_t, corspe_task: task_t, kcd_addr_begin: *vm_address_t, kcd_size: *u32) -> kern_return_t #foreign libc`
- `task_register_dyld_image_infos :: (task: task_t, dyld_images: dyld_kernel_image_info_array_t, dyld_imagesCnt: mach_msg_type_number_t) -> kern_return_t #foreign libc`
- `task_unregister_dyld_image_infos :: (task: task_t, dyld_images: dyld_kernel_image_info_array_t, dyld_imagesCnt: mach_msg_type_number_t) -> kern_return_t #foreign libc`
- `task_get_dyld_image_infos :: (task: task_inspect_t, dyld_images: *dyld_kernel_image_info_array_t, dyld_imagesCnt: *mach_msg_type_number_t) -> kern_return_t #foreign libc`
- `task_register_dyld_shared_cache_image_info :: (task: task_t, dyld_cache_image: dyld_kernel_image_info_t, no_cache: u32, private_cache: u32) -> kern_return_t #foreign libc`
- `task_register_dyld_set_dyld_state :: (task: task_t, dyld_state: u8) -> kern_return_t #foreign libc`
- `task_register_dyld_get_process_state :: (task: task_t, dyld_process_state: *dyld_kernel_process_info_t) -> kern_return_t #foreign libc`
- `task_map_corpse_info_64 :: (task: task_t, corspe_task: task_t, kcd_addr_begin: *mach_vm_address_t, kcd_size: *mach_vm_size_t) -> kern_return_t #foreign libc`
- `task_inspect :: (task: task_inspect_t, flavor: task_inspect_flavor_t, info_out: task_inspect_info_t, info_outCnt: *mach_msg_type_number_t) -> kern_return_t #foreign libc`
- `__Request__task_create_t :: struct`
- `__Request__task_terminate_t :: struct`
- `__Request__task_threads_t :: struct`
- `__Request__mach_ports_register_t :: struct`
- `__Request__mach_ports_lookup_t :: struct`
- `__Request__task_info_t :: struct`
- `__Request__task_set_info_t :: struct`
- `__Request__task_suspend_t :: struct`
- `__Request__task_resume_t :: struct`
- `__Request__task_get_special_port_t :: struct`
- `__Request__task_set_special_port_t :: struct`
- `__Request__thread_create_t :: struct`
- `__Request__thread_create_running_t :: struct`
- `__Request__task_set_exception_ports_t :: struct`
- `__Request__task_get_exception_ports_t :: struct`
- `__Request__task_swap_exception_ports_t :: struct`
- `__Request__lock_set_create_t :: struct`
- `__Request__lock_set_destroy_t :: struct`
- `__Request__semaphore_create_t :: struct`
- `__Request__semaphore_destroy_t :: struct`
- `__Request__task_policy_set_t :: struct`
- `__Request__task_policy_get_t :: struct`
- `__Request__task_sample_t :: struct`
- `__Request__task_policy_t :: struct`
- `__Request__task_set_emulation_t :: struct`
- `__Request__task_get_emulation_vector_t :: struct`
- `__Request__task_set_emulation_vector_t :: struct`
- `__Request__task_set_ras_pc_t :: struct`
- `__Request__task_zone_info_t :: struct`
- `__Request__task_assign_t :: struct`
- `__Request__task_assign_default_t :: struct`
- `__Request__task_get_assignment_t :: struct`
- `__Request__task_set_policy_t :: struct`
- `__Request__task_get_state_t :: struct`
- `__Request__task_set_state_t :: struct`
- `__Request__task_set_phys_footprint_limit_t :: struct`
- `__Request__task_suspend2_t :: struct`
- `__Request__task_resume2_t :: struct`
- `__Request__task_purgable_info_t :: struct`
- `__Request__task_get_mach_voucher_t :: struct`
- `__Request__task_set_mach_voucher_t :: struct`
- `__Request__task_swap_mach_voucher_t :: struct`
- `__Request__task_generate_corpse_t :: struct`
- `__Request__task_map_corpse_info_t :: struct`
- `__Request__task_register_dyld_image_infos_t :: struct`
- `__Request__task_unregister_dyld_image_infos_t :: struct`
- `__Request__task_get_dyld_image_infos_t :: struct`
- `__Request__task_register_dyld_shared_cache_image_info_t :: struct`
- `__Request__task_register_dyld_set_dyld_state_t :: struct`
- `__Request__task_register_dyld_get_process_state_t :: struct`
- `__Request__task_map_corpse_info_64_t :: struct`
- `__Request__task_inspect_t :: struct`
- `__RequestUnion__task_subsystem :: struct`
- `__Reply__task_create_t :: struct`
- `__Reply__task_terminate_t :: struct`
- `__Reply__task_threads_t :: struct`
- `__Reply__mach_ports_register_t :: struct`
- `__Reply__mach_ports_lookup_t :: struct`
- `__Reply__task_info_t :: struct`
- `__Reply__task_set_info_t :: struct`
- `__Reply__task_suspend_t :: struct`
- `__Reply__task_resume_t :: struct`
- `__Reply__task_get_special_port_t :: struct`
- `__Reply__task_set_special_port_t :: struct`
- `__Reply__thread_create_t :: struct`
- `__Reply__thread_create_running_t :: struct`
- `__Reply__task_set_exception_ports_t :: struct`
- `__Reply__task_get_exception_ports_t :: struct`
- `__Reply__task_swap_exception_ports_t :: struct`
- `__Reply__lock_set_create_t :: struct`
- `__Reply__lock_set_destroy_t :: struct`
- `__Reply__semaphore_create_t :: struct`
- `__Reply__semaphore_destroy_t :: struct`
- `__Reply__task_policy_set_t :: struct`
- `__Reply__task_policy_get_t :: struct`
- `__Reply__task_sample_t :: struct`
- `__Reply__task_policy_t :: struct`
- `__Reply__task_set_emulation_t :: struct`
- `__Reply__task_get_emulation_vector_t :: struct`
- `__Reply__task_set_emulation_vector_t :: struct`
- `__Reply__task_set_ras_pc_t :: struct`
- `__Reply__task_zone_info_t :: struct`
- `__Reply__task_assign_t :: struct`
- `__Reply__task_assign_default_t :: struct`
- `__Reply__task_get_assignment_t :: struct`
- `__Reply__task_set_policy_t :: struct`
- `__Reply__task_get_state_t :: struct`
- `__Reply__task_set_state_t :: struct`
- `__Reply__task_set_phys_footprint_limit_t :: struct`
- `__Reply__task_suspend2_t :: struct`
- `__Reply__task_resume2_t :: struct`
- `__Reply__task_purgable_info_t :: struct`
- `__Reply__task_get_mach_voucher_t :: struct`
- `__Reply__task_set_mach_voucher_t :: struct`
- `__Reply__task_swap_mach_voucher_t :: struct`
- `__Reply__task_generate_corpse_t :: struct`
- `__Reply__task_map_corpse_info_t :: struct`
- `__Reply__task_register_dyld_image_infos_t :: struct`
- `__Reply__task_unregister_dyld_image_infos_t :: struct`
- `__Reply__task_get_dyld_image_infos_t :: struct`
- `__Reply__task_register_dyld_shared_cache_image_info_t :: struct`
- `__Reply__task_register_dyld_set_dyld_state_t :: struct`
- `__Reply__task_register_dyld_get_process_state_t :: struct`
- `__Reply__task_map_corpse_info_64_t :: struct`
- `__Reply__task_inspect_t :: struct`
- `__ReplyUnion__task_subsystem :: struct`
- `kqueue :: () -> s32 #foreign libc`
- `kevent64 :: (kq: s32, changelist: *Kevent64, nchanges: s32, eventlist: *Kevent64, nevents: s32, flags: Kevent_Call_Flags, timeout: *timespec) -> s32 #foreign libc`
- `Kevent_Call_Flags :: enum_flags u32`
- `Kevent64 :: struct`
- `Kevent_Filter :: enum s16`
- `Kevent_Flags :: enum_flags u16`
- `Kevent_Fflags :: enum_flags u32`
- `to_string :: (cfstring: CFStringRef) -> string`
- `cfstring_from_string :: (str: string) -> CFStringRef`
- `FSEventStreamRef :: *struct {}`
- `ConstFSEventStreamRef :: FSEventStreamRef`
- `FSEventStreamEventFlags :: u32`
- `FSEventStreamCreateFlags :: u32`
- `FSEventStreamEventId :: u64`
- `kFSEventStreamEventIdSinceNow : u64`
- `kFSEventStreamCreateFlagUseCFTypes :: 0x00000001`
- `kFSEventStreamCreateFlagFileEvents :: 0x00000010`
- `kFSEventStreamEventFlagMustScanSubDirs :: 0x00000001`
- `kFSEventStreamEventFlagItemCreated :: 0x00000100`
- `kFSEventStreamEventFlagItemRemoved :: 0x00000200`
- `kFSEventStreamEventFlagItemInodeMetaMod :: 0x00000400`
- `kFSEventStreamEventFlagItemRenamed :: 0x00000800`
- `kFSEventStreamEventFlagItemModified :: 0x00001000`
- `kFSEventStreamEventFlagItemFinderInfoMod :: 0x00002000`
- `kFSEventStreamEventFlagChangeOwner :: 0x00004000`
- `kFSEventStreamEventFlagItemIsFile :: 0x00010000`
- `kFSEventStreamEventFlagItemIsDir :: 0x00020000`
- `kFSEventStreamEventFlagItemIsSymlink :: 0x00040000`
- `FSEventStreamContext :: struct`
- `FSEventStreamCallback :: #type (streamRef: ConstFSEventStreamRef, clientCallBackInfo: *void, numEvents: u64, eventPaths: *void, eventFlags: *FSEventStreamEventFlags, eventIds: *FSEventStreamEventId) #c_call`
- `FSEventStreamCreate :: (allocator: CFAllocatorRef, callback: FSEventStreamCallback, _context: *FSEventStreamContext, pathsToWatch: CFArrayRef, sinceWhen: FSEventStreamEventId, latency: CFTimeInterval, flags: FSEventStreamCreateFlags) -> ConstFSEventStreamRef #foreign CoreServices`
- `FSEventStreamScheduleWithRunLoop :: (streamRef: ConstFSEventStreamRef, runLoop: CFRunLoopRef, runLoopMode: CFStringRef) #foreign CoreServices`
- `FSEventStreamStart :: (streamRef: ConstFSEventStreamRef) -> bool #foreign CoreServices`
- `FSEventStreamStop :: (streamRef: ConstFSEventStreamRef) #foreign CoreServices`
- `FSEventStreamInvalidate :: (streamRef: ConstFSEventStreamRef) #foreign CoreServices`
- `FSEventStreamUnscheduleFromRunLoop :: (streamRef: ConstFSEventStreamRef, runLoop: CFRunLoopRef, runLoopMode: CFStringRef) #foreign CoreServices`
- `FSEventStreamRelease :: (streamRef: ConstFSEventStreamRef) #foreign CoreServices`
- `VM_MAKE_TAG :: (val: s32) -> s32`

- `NDR_record: NDR_record_t #elsewhere libc`
- `kCFAbsoluteTimeIntervalSince1904: CFTimeInterval #elsewhere corefoundation`
- `kCFAbsoluteTimeIntervalSince1970: CFTimeInterval #elsewhere corefoundation`
- `kCFAllocatorDefault: CFAllocatorRef #elsewhere corefoundation`
- `kCFAllocatorMalloc: CFAllocatorRef #elsewhere corefoundation`
- `kCFAllocatorMallocZone: CFAllocatorRef #elsewhere corefoundation`
- `kCFAllocatorNull: CFAllocatorRef #elsewhere corefoundation`
- `kCFAllocatorSystemDefault: CFAllocatorRef #elsewhere corefoundation`
- `kCFAllocatorUseContext: CFAllocatorRef #elsewhere corefoundation`
- `kCFBooleanFalse: CFBooleanRef #elsewhere corefoundation`
- `kCFBooleanTrue: CFBooleanRef #elsewhere corefoundation`
- `kCFBuddhistCalendar: CFCalendarIdentifier #elsewhere corefoundation`
- `kCFBundleDevelopmentRegionKey: CFStringRef #elsewhere corefoundation`
- `kCFBundleExecutableKey: CFStringRef #elsewhere corefoundation`
- `kCFBundleIdentifierKey: CFStringRef #elsewhere corefoundation`
- `kCFBundleInfoDictionaryVersionKey: CFStringRef #elsewhere corefoundation`
- `kCFBundleLocalizationsKey: CFStringRef #elsewhere corefoundation`
- `kCFBundleNameKey: CFStringRef #elsewhere corefoundation`
- `kCFBundleVersionKey: CFStringRef #elsewhere corefoundation`
- `kCFChineseCalendar: CFCalendarIdentifier #elsewhere corefoundation`
- `kCFCopyStringDictionaryKeyCallBacks: CFDictionaryKeyCallBacks #elsewhere corefoundation`
- `kCFDateFormatterAMSymbol: CFDateFormatterKey #elsewhere corefoundation`
- `kCFDateFormatterCalendar: CFDateFormatterKey #elsewhere corefoundation`
- `kCFDateFormatterCalendarName: CFDateFormatterKey #elsewhere corefoundation`
- `kCFDateFormatterDefaultDate: CFDateFormatterKey #elsewhere corefoundation`
- `kCFDateFormatterDefaultFormat: CFDateFormatterKey #elsewhere corefoundation`
- `kCFDateFormatterDoesRelativeDateFormattingKey: CFDateFormatterKey #elsewhere corefoundation`
- `kCFDateFormatterEraSymbols: CFDateFormatterKey #elsewhere corefoundation`
- `kCFDateFormatterGregorianStartDate: CFDateFormatterKey #elsewhere corefoundation`
- `kCFDateFormatterIsLenient: CFDateFormatterKey #elsewhere corefoundation`
- `kCFDateFormatterLongEraSymbols: CFDateFormatterKey #elsewhere corefoundation`
- `kCFDateFormatterMonthSymbols: CFDateFormatterKey #elsewhere corefoundation`
- `kCFDateFormatterPMSymbol: CFDateFormatterKey #elsewhere corefoundation`
- `kCFDateFormatterQuarterSymbols: CFDateFormatterKey #elsewhere corefoundation`
- `kCFDateFormatterShortMonthSymbols: CFDateFormatterKey #elsewhere corefoundation`
- `kCFDateFormatterShortQuarterSymbols: CFDateFormatterKey #elsewhere corefoundation`
- `kCFDateFormatterShortStandaloneMonthSymbols: CFDateFormatterKey #elsewhere corefoundation`
- `kCFDateFormatterShortStandaloneQuarterSymbols: CFDateFormatterKey #elsewhere corefoundation`
- `kCFDateFormatterShortStandaloneWeekdaySymbols: CFDateFormatterKey #elsewhere corefoundation`
- `kCFDateFormatterShortWeekdaySymbols: CFDateFormatterKey #elsewhere corefoundation`
- `kCFDateFormatterStandaloneMonthSymbols: CFDateFormatterKey #elsewhere corefoundation`
- `kCFDateFormatterStandaloneQuarterSymbols: CFDateFormatterKey #elsewhere corefoundation`
- `kCFDateFormatterStandaloneWeekdaySymbols: CFDateFormatterKey #elsewhere corefoundation`
- `kCFDateFormatterTimeZone: CFDateFormatterKey #elsewhere corefoundation`
- `kCFDateFormatterTwoDigitStartDate: CFDateFormatterKey #elsewhere corefoundation`
- `kCFDateFormatterVeryShortMonthSymbols: CFDateFormatterKey #elsewhere corefoundation`
- `kCFDateFormatterVeryShortStandaloneMonthSymbols: CFDateFormatterKey #elsewhere corefoundation`
- `kCFDateFormatterVeryShortStandaloneWeekdaySymbols: CFDateFormatterKey #elsewhere corefoundation`
- `kCFDateFormatterVeryShortWeekdaySymbols: CFDateFormatterKey #elsewhere corefoundation`
- `kCFDateFormatterWeekdaySymbols: CFDateFormatterKey #elsewhere corefoundation`
- `kCFErrorDescriptionKey: CFStringRef #elsewhere corefoundation`
- `kCFErrorDomainCocoa: CFErrorDomain #elsewhere corefoundation`
- `kCFErrorDomainMach: CFErrorDomain #elsewhere corefoundation`
- `kCFErrorDomainOSStatus: CFErrorDomain #elsewhere corefoundation`
- `kCFErrorDomainPOSIX: CFErrorDomain #elsewhere corefoundation`
- `kCFErrorFilePathKey: CFStringRef #elsewhere corefoundation`
- `kCFErrorLocalizedDescriptionKey: CFStringRef #elsewhere corefoundation`
- `kCFErrorLocalizedFailureKey: CFStringRef #elsewhere corefoundation`
- `kCFErrorLocalizedFailureReasonKey: CFStringRef #elsewhere corefoundation`
- `kCFErrorLocalizedRecoverySuggestionKey: CFStringRef #elsewhere corefoundation`
- `kCFErrorURLKey: CFStringRef #elsewhere corefoundation`
- `kCFErrorUnderlyingErrorKey: CFStringRef #elsewhere corefoundation`
- `kCFGregorianCalendar: CFCalendarIdentifier #elsewhere corefoundation`
- `kCFHebrewCalendar: CFCalendarIdentifier #elsewhere corefoundation`
- `kCFISO8601Calendar: CFCalendarIdentifier #elsewhere corefoundation`
- `kCFIndianCalendar: CFCalendarIdentifier #elsewhere corefoundation`
- `kCFIslamicCalendar: CFCalendarIdentifier #elsewhere corefoundation`
- `kCFIslamicCivilCalendar: CFCalendarIdentifier #elsewhere corefoundation`
- `kCFIslamicTabularCalendar: CFCalendarIdentifier #elsewhere corefoundation`
- `kCFIslamicUmmAlQuraCalendar: CFCalendarIdentifier #elsewhere corefoundation`
- `kCFJapaneseCalendar: CFCalendarIdentifier #elsewhere corefoundation`
- `kCFLocaleAlternateQuotationBeginDelimiterKey: CFLocaleKey #elsewhere corefoundation`
- `kCFLocaleAlternateQuotationEndDelimiterKey: CFLocaleKey #elsewhere corefoundation`
- `kCFLocaleCalendar: CFLocaleKey #elsewhere corefoundation`
- `kCFLocaleCalendarIdentifier: CFLocaleKey #elsewhere corefoundation`
- `kCFLocaleCollationIdentifier: CFLocaleKey #elsewhere corefoundation`
- `kCFLocaleCollatorIdentifier: CFLocaleKey #elsewhere corefoundation`
- `kCFLocaleCountryCode: CFLocaleKey #elsewhere corefoundation`
- `kCFLocaleCurrencyCode: CFLocaleKey #elsewhere corefoundation`
- `kCFLocaleCurrencySymbol: CFLocaleKey #elsewhere corefoundation`
- `kCFLocaleCurrentLocaleDidChangeNotification: CFNotificationName #elsewhere corefoundation`
- `kCFLocaleDecimalSeparator: CFLocaleKey #elsewhere corefoundation`
- `kCFLocaleExemplarCharacterSet: CFLocaleKey #elsewhere corefoundation`
- `kCFLocaleGroupingSeparator: CFLocaleKey #elsewhere corefoundation`
- `kCFLocaleIdentifier: CFLocaleKey #elsewhere corefoundation`
- `kCFLocaleLanguageCode: CFLocaleKey #elsewhere corefoundation`
- `kCFLocaleMeasurementSystem: CFLocaleKey #elsewhere corefoundation`
- `kCFLocaleQuotationBeginDelimiterKey: CFLocaleKey #elsewhere corefoundation`
- `kCFLocaleQuotationEndDelimiterKey: CFLocaleKey #elsewhere corefoundation`
- `kCFLocaleScriptCode: CFLocaleKey #elsewhere corefoundation`
- `kCFLocaleUsesMetricSystem: CFLocaleKey #elsewhere corefoundation`
- `kCFLocaleVariantCode: CFLocaleKey #elsewhere corefoundation`
- `kCFNull: CFNullRef #elsewhere corefoundation`
- `kCFNumberFormatterAlwaysShowDecimalSeparator: CFNumberFormatterKey #elsewhere corefoundation`
- `kCFNumberFormatterCurrencyCode: CFNumberFormatterKey #elsewhere corefoundation`
- `kCFNumberFormatterCurrencyDecimalSeparator: CFNumberFormatterKey #elsewhere corefoundation`
- `kCFNumberFormatterCurrencyGroupingSeparator: CFNumberFormatterKey #elsewhere corefoundation`
- `kCFNumberFormatterCurrencySymbol: CFNumberFormatterKey #elsewhere corefoundation`
- `kCFNumberFormatterDecimalSeparator: CFNumberFormatterKey #elsewhere corefoundation`
- `kCFNumberFormatterDefaultFormat: CFNumberFormatterKey #elsewhere corefoundation`
- `kCFNumberFormatterExponentSymbol: CFNumberFormatterKey #elsewhere corefoundation`
- `kCFNumberFormatterFormatWidth: CFNumberFormatterKey #elsewhere corefoundation`
- `kCFNumberFormatterGroupingSeparator: CFNumberFormatterKey #elsewhere corefoundation`
- `kCFNumberFormatterGroupingSize: CFNumberFormatterKey #elsewhere corefoundation`
- `kCFNumberFormatterInfinitySymbol: CFNumberFormatterKey #elsewhere corefoundation`
- `kCFNumberFormatterInternationalCurrencySymbol: CFNumberFormatterKey #elsewhere corefoundation`
- `kCFNumberFormatterIsLenient: CFNumberFormatterKey #elsewhere corefoundation`
- `kCFNumberFormatterMaxFractionDigits: CFNumberFormatterKey #elsewhere corefoundation`
- `kCFNumberFormatterMaxIntegerDigits: CFNumberFormatterKey #elsewhere corefoundation`
- `kCFNumberFormatterMaxSignificantDigits: CFNumberFormatterKey #elsewhere corefoundation`
- `kCFNumberFormatterMinFractionDigits: CFNumberFormatterKey #elsewhere corefoundation`
- `kCFNumberFormatterMinIntegerDigits: CFNumberFormatterKey #elsewhere corefoundation`
- `kCFNumberFormatterMinSignificantDigits: CFNumberFormatterKey #elsewhere corefoundation`
- `kCFNumberFormatterMinusSign: CFNumberFormatterKey #elsewhere corefoundation`
- `kCFNumberFormatterMultiplier: CFNumberFormatterKey #elsewhere corefoundation`
- `kCFNumberFormatterNaNSymbol: CFNumberFormatterKey #elsewhere corefoundation`
- `kCFNumberFormatterNegativePrefix: CFNumberFormatterKey #elsewhere corefoundation`
- `kCFNumberFormatterNegativeSuffix: CFNumberFormatterKey #elsewhere corefoundation`
- `kCFNumberFormatterPaddingCharacter: CFNumberFormatterKey #elsewhere corefoundation`
- `kCFNumberFormatterPaddingPosition: CFNumberFormatterKey #elsewhere corefoundation`
- `kCFNumberFormatterPerMillSymbol: CFNumberFormatterKey #elsewhere corefoundation`
- `kCFNumberFormatterPercentSymbol: CFNumberFormatterKey #elsewhere corefoundation`
- `kCFNumberFormatterPlusSign: CFNumberFormatterKey #elsewhere corefoundation`
- `kCFNumberFormatterPositivePrefix: CFNumberFormatterKey #elsewhere corefoundation`
- `kCFNumberFormatterPositiveSuffix: CFNumberFormatterKey #elsewhere corefoundation`
- `kCFNumberFormatterRoundingIncrement: CFNumberFormatterKey #elsewhere corefoundation`
- `kCFNumberFormatterRoundingMode: CFNumberFormatterKey #elsewhere corefoundation`
- `kCFNumberFormatterSecondaryGroupingSize: CFNumberFormatterKey #elsewhere corefoundation`
- `kCFNumberFormatterUseGroupingSeparator: CFNumberFormatterKey #elsewhere corefoundation`
- `kCFNumberFormatterUseSignificantDigits: CFNumberFormatterKey #elsewhere corefoundation`
- `kCFNumberFormatterZeroSymbol: CFNumberFormatterKey #elsewhere corefoundation`
- `kCFNumberNaN: CFNumberRef #elsewhere corefoundation`
- `kCFNumberNegativeInfinity: CFNumberRef #elsewhere corefoundation`
- `kCFNumberPositiveInfinity: CFNumberRef #elsewhere corefoundation`
- `kCFPersianCalendar: CFCalendarIdentifier #elsewhere corefoundation`
- `kCFPlugInDynamicRegisterFunctionKey: CFStringRef #elsewhere corefoundation`
- `kCFPlugInDynamicRegistrationKey: CFStringRef #elsewhere corefoundation`
- `kCFPlugInFactoriesKey: CFStringRef #elsewhere corefoundation`
- `kCFPlugInTypesKey: CFStringRef #elsewhere corefoundation`
- `kCFPlugInUnloadFunctionKey: CFStringRef #elsewhere corefoundation`
- `kCFPreferencesAnyApplication: CFStringRef #elsewhere corefoundation`
- `kCFPreferencesAnyHost: CFStringRef #elsewhere corefoundation`
- `kCFPreferencesAnyUser: CFStringRef #elsewhere corefoundation`
- `kCFPreferencesCurrentApplication: CFStringRef #elsewhere corefoundation`
- `kCFPreferencesCurrentHost: CFStringRef #elsewhere corefoundation`
- `kCFPreferencesCurrentUser: CFStringRef #elsewhere corefoundation`
- `kCFRepublicOfChinaCalendar: CFCalendarIdentifier #elsewhere corefoundation`
- `kCFRunLoopCommonModes: CFRunLoopMode #elsewhere corefoundation`
- `kCFRunLoopDefaultMode: CFRunLoopMode #elsewhere corefoundation`
- `kCFSocketCommandKey: CFStringRef #elsewhere corefoundation`
- `kCFSocketErrorKey: CFStringRef #elsewhere corefoundation`
- `kCFSocketNameKey: CFStringRef #elsewhere corefoundation`
- `kCFSocketRegisterCommand: CFStringRef #elsewhere corefoundation`
- `kCFSocketResultKey: CFStringRef #elsewhere corefoundation`
- `kCFSocketRetrieveCommand: CFStringRef #elsewhere corefoundation`
- `kCFSocketValueKey: CFStringRef #elsewhere corefoundation`
- `kCFStreamPropertyAppendToFile: CFStreamPropertyKey #elsewhere corefoundation`
- `kCFStreamPropertyDataWritten: CFStreamPropertyKey #elsewhere corefoundation`
- `kCFStreamPropertyFileCurrentOffset: CFStreamPropertyKey #elsewhere corefoundation`
- `kCFStreamPropertySocketNativeHandle: CFStreamPropertyKey #elsewhere corefoundation`
- `kCFStreamPropertySocketRemoteHostName: CFStreamPropertyKey #elsewhere corefoundation`
- `kCFStreamPropertySocketRemotePortNumber: CFStreamPropertyKey #elsewhere corefoundation`
- `kCFStringTransformFullwidthHalfwidth: CFStringRef #elsewhere corefoundation`
- `kCFStringTransformHiraganaKatakana: CFStringRef #elsewhere corefoundation`
- `kCFStringTransformLatinArabic: CFStringRef #elsewhere corefoundation`
- `kCFStringTransformLatinCyrillic: CFStringRef #elsewhere corefoundation`
- `kCFStringTransformLatinGreek: CFStringRef #elsewhere corefoundation`
- `kCFStringTransformLatinHangul: CFStringRef #elsewhere corefoundation`
- `kCFStringTransformLatinHebrew: CFStringRef #elsewhere corefoundation`
- `kCFStringTransformLatinHiragana: CFStringRef #elsewhere corefoundation`
- `kCFStringTransformLatinKatakana: CFStringRef #elsewhere corefoundation`
- `kCFStringTransformLatinThai: CFStringRef #elsewhere corefoundation`
- `kCFStringTransformMandarinLatin: CFStringRef #elsewhere corefoundation`
- `kCFStringTransformStripCombiningMarks: CFStringRef #elsewhere corefoundation`
- `kCFStringTransformStripDiacritics: CFStringRef #elsewhere corefoundation`
- `kCFStringTransformToLatin: CFStringRef #elsewhere corefoundation`
- `kCFStringTransformToUnicodeName: CFStringRef #elsewhere corefoundation`
- `kCFStringTransformToXMLHex: CFStringRef #elsewhere corefoundation`
- `kCFTimeZoneSystemTimeZoneDidChangeNotification: CFNotificationName #elsewhere corefoundation`
- `kCFTypeArrayCallBacks: CFArrayCallBacks #elsewhere corefoundation`
- `kCFTypeDictionaryKeyCallBacks: CFDictionaryKeyCallBacks #elsewhere corefoundation`
- `kCFTypeDictionaryValueCallBacks: CFDictionaryValueCallBacks #elsewhere corefoundation`
- `kCFURLAddedToDirectoryDateKey: CFStringRef #elsewhere corefoundation`
- `kCFURLApplicationIsScriptableKey: CFStringRef #elsewhere corefoundation`
- `kCFURLAttributeModificationDateKey: CFStringRef #elsewhere corefoundation`
- `kCFURLCanonicalPathKey: CFStringRef #elsewhere corefoundation`
- `kCFURLContentAccessDateKey: CFStringRef #elsewhere corefoundation`
- `kCFURLContentModificationDateKey: CFStringRef #elsewhere corefoundation`
- `kCFURLCreationDateKey: CFStringRef #elsewhere corefoundation`
- `kCFURLCustomIconKey: CFStringRef #elsewhere corefoundation`
- `kCFURLDocumentIdentifierKey: CFStringRef #elsewhere corefoundation`
- `kCFURLEffectiveIconKey: CFStringRef #elsewhere corefoundation`
- `kCFURLFileAllocatedSizeKey: CFStringRef #elsewhere corefoundation`
- `kCFURLFileDirectoryContents: CFStringRef #elsewhere corefoundation`
- `kCFURLFileExists: CFStringRef #elsewhere corefoundation`
- `kCFURLFileLastModificationTime: CFStringRef #elsewhere corefoundation`
- `kCFURLFileLength: CFStringRef #elsewhere corefoundation`
- `kCFURLFileOwnerID: CFStringRef #elsewhere corefoundation`
- `kCFURLFilePOSIXMode: CFStringRef #elsewhere corefoundation`
- `kCFURLFileProtectionComplete: CFStringRef #elsewhere corefoundation`
- `kCFURLFileProtectionCompleteUnlessOpen: CFStringRef #elsewhere corefoundation`
- `kCFURLFileProtectionCompleteUntilFirstUserAuthentication: CFStringRef #elsewhere corefoundation`
- `kCFURLFileProtectionKey: CFStringRef #elsewhere corefoundation`
- `kCFURLFileProtectionNone: CFStringRef #elsewhere corefoundation`
- `kCFURLFileResourceIdentifierKey: CFStringRef #elsewhere corefoundation`
- `kCFURLFileResourceTypeBlockSpecial: CFStringRef #elsewhere corefoundation`
- `kCFURLFileResourceTypeCharacterSpecial: CFStringRef #elsewhere corefoundation`
- `kCFURLFileResourceTypeDirectory: CFStringRef #elsewhere corefoundation`
- `kCFURLFileResourceTypeKey: CFStringRef #elsewhere corefoundation`
- `kCFURLFileResourceTypeNamedPipe: CFStringRef #elsewhere corefoundation`
- `kCFURLFileResourceTypeRegular: CFStringRef #elsewhere corefoundation`
- `kCFURLFileResourceTypeSocket: CFStringRef #elsewhere corefoundation`
- `kCFURLFileResourceTypeSymbolicLink: CFStringRef #elsewhere corefoundation`
- `kCFURLFileResourceTypeUnknown: CFStringRef #elsewhere corefoundation`
- `kCFURLFileSecurityKey: CFStringRef #elsewhere corefoundation`
- `kCFURLFileSizeKey: CFStringRef #elsewhere corefoundation`
- `kCFURLGenerationIdentifierKey: CFStringRef #elsewhere corefoundation`
- `kCFURLHTTPStatusCode: CFStringRef #elsewhere corefoundation`
- `kCFURLHTTPStatusLine: CFStringRef #elsewhere corefoundation`
- `kCFURLHasHiddenExtensionKey: CFStringRef #elsewhere corefoundation`
- `kCFURLIsAliasFileKey: CFStringRef #elsewhere corefoundation`
- `kCFURLIsApplicationKey: CFStringRef #elsewhere corefoundation`
- `kCFURLIsDirectoryKey: CFStringRef #elsewhere corefoundation`
- `kCFURLIsExcludedFromBackupKey: CFStringRef #elsewhere corefoundation`
- `kCFURLIsExecutableKey: CFStringRef #elsewhere corefoundation`
- `kCFURLIsHiddenKey: CFStringRef #elsewhere corefoundation`
- `kCFURLIsMountTriggerKey: CFStringRef #elsewhere corefoundation`
- `kCFURLIsPackageKey: CFStringRef #elsewhere corefoundation`
- `kCFURLIsReadableKey: CFStringRef #elsewhere corefoundation`
- `kCFURLIsRegularFileKey: CFStringRef #elsewhere corefoundation`
- `kCFURLIsSymbolicLinkKey: CFStringRef #elsewhere corefoundation`
- `kCFURLIsSystemImmutableKey: CFStringRef #elsewhere corefoundation`
- `kCFURLIsUbiquitousItemKey: CFStringRef #elsewhere corefoundation`
- `kCFURLIsUserImmutableKey: CFStringRef #elsewhere corefoundation`
- `kCFURLIsVolumeKey: CFStringRef #elsewhere corefoundation`
- `kCFURLIsWritableKey: CFStringRef #elsewhere corefoundation`
- `kCFURLKeysOfUnsetValuesKey: CFStringRef #elsewhere corefoundation`
- `kCFURLLabelColorKey: CFStringRef #elsewhere corefoundation`
- `kCFURLLabelNumberKey: CFStringRef #elsewhere corefoundation`
- `kCFURLLinkCountKey: CFStringRef #elsewhere corefoundation`
- `kCFURLLocalizedLabelKey: CFStringRef #elsewhere corefoundation`
- `kCFURLLocalizedNameKey: CFStringRef #elsewhere corefoundation`
- `kCFURLLocalizedTypeDescriptionKey: CFStringRef #elsewhere corefoundation`
- `kCFURLNameKey: CFStringRef #elsewhere corefoundation`
- `kCFURLParentDirectoryURLKey: CFStringRef #elsewhere corefoundation`
- `kCFURLPathKey: CFStringRef #elsewhere corefoundation`
- `kCFURLPreferredIOBlockSizeKey: CFStringRef #elsewhere corefoundation`
- `kCFURLQuarantinePropertiesKey: CFStringRef #elsewhere corefoundation`
- `kCFURLTagNamesKey: CFStringRef #elsewhere corefoundation`
- `kCFURLTotalFileAllocatedSizeKey: CFStringRef #elsewhere corefoundation`
- `kCFURLTotalFileSizeKey: CFStringRef #elsewhere corefoundation`
- `kCFURLTypeIdentifierKey: CFStringRef #elsewhere corefoundation`
- `kCFURLUbiquitousItemDownloadingErrorKey: CFStringRef #elsewhere corefoundation`
- `kCFURLUbiquitousItemDownloadingStatusCurrent: CFStringRef #elsewhere corefoundation`
- `kCFURLUbiquitousItemDownloadingStatusDownloaded: CFStringRef #elsewhere corefoundation`
- `kCFURLUbiquitousItemDownloadingStatusKey: CFStringRef #elsewhere corefoundation`
- `kCFURLUbiquitousItemDownloadingStatusNotDownloaded: CFStringRef #elsewhere corefoundation`
- `kCFURLUbiquitousItemHasUnresolvedConflictsKey: CFStringRef #elsewhere corefoundation`
- `kCFURLUbiquitousItemIsDownloadedKey: CFStringRef #elsewhere corefoundation`
- `kCFURLUbiquitousItemIsDownloadingKey: CFStringRef #elsewhere corefoundation`
- `kCFURLUbiquitousItemIsUploadedKey: CFStringRef #elsewhere corefoundation`
- `kCFURLUbiquitousItemIsUploadingKey: CFStringRef #elsewhere corefoundation`
- `kCFURLUbiquitousItemPercentDownloadedKey: CFStringRef #elsewhere corefoundation`
- `kCFURLUbiquitousItemPercentUploadedKey: CFStringRef #elsewhere corefoundation`
- `kCFURLUbiquitousItemUploadingErrorKey: CFStringRef #elsewhere corefoundation`
- `kCFURLVolumeAvailableCapacityForImportantUsageKey: CFStringRef #elsewhere corefoundation`
- `kCFURLVolumeAvailableCapacityForOpportunisticUsageKey: CFStringRef #elsewhere corefoundation`
- `kCFURLVolumeAvailableCapacityKey: CFStringRef #elsewhere corefoundation`
- `kCFURLVolumeCreationDateKey: CFStringRef #elsewhere corefoundation`
- `kCFURLVolumeIdentifierKey: CFStringRef #elsewhere corefoundation`
- `kCFURLVolumeIsAutomountedKey: CFStringRef #elsewhere corefoundation`
- `kCFURLVolumeIsBrowsableKey: CFStringRef #elsewhere corefoundation`
- `kCFURLVolumeIsEjectableKey: CFStringRef #elsewhere corefoundation`
- `kCFURLVolumeIsEncryptedKey: CFStringRef #elsewhere corefoundation`
- `kCFURLVolumeIsInternalKey: CFStringRef #elsewhere corefoundation`
- `kCFURLVolumeIsJournalingKey: CFStringRef #elsewhere corefoundation`
- `kCFURLVolumeIsLocalKey: CFStringRef #elsewhere corefoundation`
- `kCFURLVolumeIsReadOnlyKey: CFStringRef #elsewhere corefoundation`
- `kCFURLVolumeIsRemovableKey: CFStringRef #elsewhere corefoundation`
- `kCFURLVolumeIsRootFileSystemKey: CFStringRef #elsewhere corefoundation`
- `kCFURLVolumeLocalizedFormatDescriptionKey: CFStringRef #elsewhere corefoundation`
- `kCFURLVolumeLocalizedNameKey: CFStringRef #elsewhere corefoundation`
- `kCFURLVolumeMaximumFileSizeKey: CFStringRef #elsewhere corefoundation`
- `kCFURLVolumeNameKey: CFStringRef #elsewhere corefoundation`
- `kCFURLVolumeResourceCountKey: CFStringRef #elsewhere corefoundation`
- `kCFURLVolumeSupportsAccessPermissionsKey: CFStringRef #elsewhere corefoundation`
- `kCFURLVolumeSupportsAdvisoryFileLockingKey: CFStringRef #elsewhere corefoundation`
- `kCFURLVolumeSupportsCasePreservedNamesKey: CFStringRef #elsewhere corefoundation`
- `kCFURLVolumeSupportsCaseSensitiveNamesKey: CFStringRef #elsewhere corefoundation`
- `kCFURLVolumeSupportsCompressionKey: CFStringRef #elsewhere corefoundation`
- `kCFURLVolumeSupportsExclusiveRenamingKey: CFStringRef #elsewhere corefoundation`
- `kCFURLVolumeSupportsExtendedSecurityKey: CFStringRef #elsewhere corefoundation`
- `kCFURLVolumeSupportsFileCloningKey: CFStringRef #elsewhere corefoundation`
- `kCFURLVolumeSupportsHardLinksKey: CFStringRef #elsewhere corefoundation`
- `kCFURLVolumeSupportsImmutableFilesKey: CFStringRef #elsewhere corefoundation`
- `kCFURLVolumeSupportsJournalingKey: CFStringRef #elsewhere corefoundation`
- `kCFURLVolumeSupportsPersistentIDsKey: CFStringRef #elsewhere corefoundation`
- `kCFURLVolumeSupportsRenamingKey: CFStringRef #elsewhere corefoundation`
- `kCFURLVolumeSupportsRootDirectoryDatesKey: CFStringRef #elsewhere corefoundation`
- `kCFURLVolumeSupportsSparseFilesKey: CFStringRef #elsewhere corefoundation`
- `kCFURLVolumeSupportsSwapRenamingKey: CFStringRef #elsewhere corefoundation`
- `kCFURLVolumeSupportsSymbolicLinksKey: CFStringRef #elsewhere corefoundation`
- `kCFURLVolumeSupportsVolumeSizesKey: CFStringRef #elsewhere corefoundation`
- `kCFURLVolumeSupportsZeroRunsKey: CFStringRef #elsewhere corefoundation`
- `kCFURLVolumeTotalCapacityKey: CFStringRef #elsewhere corefoundation`
- `kCFURLVolumeURLForRemountingKey: CFStringRef #elsewhere corefoundation`
- `kCFURLVolumeURLKey: CFStringRef #elsewhere corefoundation`
- `kCFURLVolumeUUIDStringKey: CFStringRef #elsewhere corefoundation`
- `kCFUserNotificationAlertHeaderKey: CFStringRef #elsewhere corefoundation`
- `kCFUserNotificationAlertMessageKey: CFStringRef #elsewhere corefoundation`
- `kCFUserNotificationAlternateButtonTitleKey: CFStringRef #elsewhere corefoundation`
- `kCFUserNotificationCheckBoxTitlesKey: CFStringRef #elsewhere corefoundation`
- `kCFUserNotificationDefaultButtonTitleKey: CFStringRef #elsewhere corefoundation`
- `kCFUserNotificationIconURLKey: CFStringRef #elsewhere corefoundation`
- `kCFUserNotificationLocalizationURLKey: CFStringRef #elsewhere corefoundation`
- `kCFUserNotificationOtherButtonTitleKey: CFStringRef #elsewhere corefoundation`
- `kCFUserNotificationPopUpSelectionKey: CFStringRef #elsewhere corefoundation`
- `kCFUserNotificationPopUpTitlesKey: CFStringRef #elsewhere corefoundation`
- `kCFUserNotificationProgressIndicatorValueKey: CFStringRef #elsewhere corefoundation`
- `kCFUserNotificationSoundURLKey: CFStringRef #elsewhere corefoundation`
- `kCFUserNotificationTextFieldTitlesKey: CFStringRef #elsewhere corefoundation`
- `kCFUserNotificationTextFieldValuesKey: CFStringRef #elsewhere corefoundation`

## Public type members

### macOS-specific type declarations

#### `CFComparisonResult` (enum)

- `LessThan    :: -1`
- `EqualTo     :: 0`
- `GreaterThan :: 1`
- `kCFCompareLessThan    :: LessThan`
- `kCFCompareEqualTo     :: EqualTo`
- `kCFCompareGreaterThan :: GreaterThan`

#### `CFRange` (struct)

- `location: CFIndex`
- `length:   CFIndex`

#### `CFAllocatorContext` (struct)

- `version:         CFIndex`
- `info:            *void`
- `retain:          CFAllocatorRetainCallBack`
- `release:         CFAllocatorReleaseCallBack`
- `copyDescription: CFAllocatorCopyDescriptionCallBack`
- `allocate:        CFAllocatorAllocateCallBack`
- `reallocate:      CFAllocatorReallocateCallBack`
- `deallocate:      CFAllocatorDeallocateCallBack`
- `preferredSize:   CFAllocatorPreferredSizeCallBack`

#### `CFArrayCallBacks` (struct)

- `version:         CFIndex`
- `retain:          CFArrayRetainCallBack`
- `release:         CFArrayReleaseCallBack`
- `copyDescription: CFArrayCopyDescriptionCallBack`
- `equal:           CFArrayEqualCallBack`

#### `__CFByteOrder` (enum)

- `Unknown      :: 0`
- `LittleEndian :: 1`
- `BigEndian    :: 2`
- `CFByteOrderUnknown      :: Unknown`
- `CFByteOrderLittleEndian :: LittleEndian`
- `CFByteOrderBigEndian    :: BigEndian`

#### `CFSwappedFloat32` (struct)

- `v: u32`

#### `CFSwappedFloat64` (struct)

- `v: u64`

#### `CFDictionaryKeyCallBacks` (struct)

- `version:         CFIndex`
- `retain:          CFDictionaryRetainCallBack`
- `release:         CFDictionaryReleaseCallBack`
- `copyDescription: CFDictionaryCopyDescriptionCallBack`
- `equal:           CFDictionaryEqualCallBack`
- `hash:            CFDictionaryHashCallBack`

#### `CFDictionaryValueCallBacks` (struct)

- `version:         CFIndex`
- `retain:          CFDictionaryRetainCallBack`
- `release:         CFDictionaryReleaseCallBack`
- `copyDescription: CFDictionaryCopyDescriptionCallBack`
- `equal:           CFDictionaryEqualCallBack`

#### `CFNotificationSuspensionBehavior` (enum)

- `Drop               :: 1`
- `Coalesce           :: 2`
- `Hold               :: 3`
- `DeliverImmediately :: 4`
- `CFNotificationSuspensionBehaviorDrop               :: Drop`
- `CFNotificationSuspensionBehaviorCoalesce           :: Coalesce`
- `CFNotificationSuspensionBehaviorHold               :: Hold`
- `CFNotificationSuspensionBehaviorDeliverImmediately :: DeliverImmediately`

#### `kCFNotification` (enum)

- `DeliverImmediately :: 1`
- `PostToAllSessions  :: 2`
- `kCFNotificationDeliverImmediately :: DeliverImmediately`
- `kCFNotificationPostToAllSessions  :: PostToAllSessions`

#### `CFLocaleLanguageDirection` (enum)

- `Unknown     :: 0`
- `LeftToRight :: 1`
- `RightToLeft :: 2`
- `TopToBottom :: 3`
- `BottomToTop :: 4`
- `kCFLocaleLanguageDirectionUnknown     :: Unknown`
- `kCFLocaleLanguageDirectionLeftToRight :: LeftToRight`
- `kCFLocaleLanguageDirectionRightToLeft :: RightToLeft`
- `kCFLocaleLanguageDirectionTopToBottom :: TopToBottom`
- `kCFLocaleLanguageDirectionBottomToTop :: BottomToTop`

#### `CFGregorianDate` (struct)

- `year:   s32`
- `month:  s8`
- `day:    s8`
- `hour:   s8`
- `minute: s8`
- `second: float64`

#### `CFGregorianUnits` (struct)

- `years:   s32`
- `months:  s32`
- `days:    s32`
- `hours:   s32`
- `minutes: s32`
- `seconds: float64`

#### `kCFGregorian` (enum)

- `UnitsYears   :: 1`
- `UnitsMonths  :: 2`
- `UnitsDays    :: 4`
- `UnitsHours   :: 8`
- `UnitsMinutes :: 16`
- `UnitsSeconds :: 32`
- `AllUnits     :: 16777215`
- `kCFGregorianUnitsYears   :: UnitsYears`
- `kCFGregorianUnitsMonths  :: UnitsMonths`
- `kCFGregorianUnitsDays    :: UnitsDays`
- `kCFGregorianUnitsHours   :: UnitsHours`
- `kCFGregorianUnitsMinutes :: UnitsMinutes`
- `kCFGregorianUnitsSeconds :: UnitsSeconds`
- `kCFGregorianAllUnits     :: AllUnits`

#### `CFDataSearchFlags` (enum_flags)

- `Backwards :: 0x1`
- `Anchored  :: 0x2`
- `kCFDataSearchBackwards :: Backwards`
- `kCFDataSearchAnchored  :: Anchored`

#### `CFCharacterSetPredefinedSet` (enum)

- `Control              :: 1`
- `Whitespace           :: 2`
- `WhitespaceAndNewline :: 3`
- `DecimalDigit         :: 4`
- `Letter               :: 5`
- `LowercaseLetter      :: 6`
- `UppercaseLetter      :: 7`
- `NonBase              :: 8`
- `Decomposable         :: 9`
- `AlphaNumeric         :: 10`
- `Punctuation          :: 11`
- `CapitalizedLetter    :: 13`
- `Symbol               :: 14`
- `Newline              :: 15`
- `Illegal              :: 12`
- `kCFCharacterSetControl              :: Control`
- `kCFCharacterSetWhitespace           :: Whitespace`
- `kCFCharacterSetWhitespaceAndNewline :: WhitespaceAndNewline`
- `kCFCharacterSetDecimalDigit         :: DecimalDigit`
- `kCFCharacterSetLetter               :: Letter`
- `kCFCharacterSetLowercaseLetter      :: LowercaseLetter`
- `kCFCharacterSetUppercaseLetter      :: UppercaseLetter`
- `kCFCharacterSetNonBase              :: NonBase`
- `kCFCharacterSetDecomposable         :: Decomposable`
- `kCFCharacterSetAlphaNumeric         :: AlphaNumeric`
- `kCFCharacterSetPunctuation          :: Punctuation`
- `kCFCharacterSetCapitalizedLetter    :: CapitalizedLetter`
- `kCFCharacterSetSymbol               :: Symbol`
- `kCFCharacterSetNewline              :: Newline`
- `kCFCharacterSetIllegal              :: Illegal`

#### `CFStringEncoding` (enum)

- `MacRoman      :: 0`
- `WindowsLatin1 :: 1280`
- `ISOLatin1     :: 513`
- `NextStepLatin :: 2817`
- `ASCII         :: 1536`
- `Unicode       :: 256`
- `UTF8          :: 134217984`
- `NonLossyASCII :: 3071`
- `UTF16         :: 256`
- `UTF16BE       :: 268435712`
- `UTF16LE       :: 335544576`
- `UTF32         :: 201326848`
- `UTF32BE       :: 402653440`
- `UTF32LE       :: 469762304`
- `kCFStringEncodingMacRoman      :: MacRoman`
- `kCFStringEncodingWindowsLatin1 :: WindowsLatin1`
- `kCFStringEncodingISOLatin1     :: ISOLatin1`
- `kCFStringEncodingNextStepLatin :: NextStepLatin`
- `kCFStringEncodingASCII         :: ASCII`
- `kCFStringEncodingUnicode       :: Unicode`
- `kCFStringEncodingUTF8          :: UTF8`
- `kCFStringEncodingNonLossyASCII :: NonLossyASCII`
- `kCFStringEncodingUTF16         :: UTF16`
- `kCFStringEncodingUTF16BE       :: UTF16BE`
- `kCFStringEncodingUTF16LE       :: UTF16LE`
- `kCFStringEncodingUTF32         :: UTF32`
- `kCFStringEncodingUTF32BE       :: UTF32BE`
- `kCFStringEncodingUTF32LE       :: UTF32LE`

#### `CFStringCompareFlags` (enum_flags)

- `CaseInsensitive      :: 0x1`
- `Backwards            :: 0x4`
- `Anchored             :: 0x8`
- `Nonliteral           :: 0x10`
- `Localized            :: 0x20`
- `Numerically          :: 0x40`
- `DiacriticInsensitive :: 0x80`
- `WidthInsensitive     :: 0x100`
- `ForcedOrdering       :: 0x200`
- `kCFCompareCaseInsensitive      :: CaseInsensitive`
- `kCFCompareBackwards            :: Backwards`
- `kCFCompareAnchored             :: Anchored`
- `kCFCompareNonliteral           :: Nonliteral`
- `kCFCompareLocalized            :: Localized`
- `kCFCompareNumerically          :: Numerically`
- `kCFCompareDiacriticInsensitive :: DiacriticInsensitive`
- `kCFCompareWidthInsensitive     :: WidthInsensitive`
- `kCFCompareForcedOrdering       :: ForcedOrdering`

#### `CFStringNormalizationForm` (enum)

- `D  :: 0`
- `KD :: 1`
- `C  :: 2`
- `KC :: 3`
- `kCFStringNormalizationFormD  :: D`
- `kCFStringNormalizationFormKD :: KD`
- `kCFStringNormalizationFormC  :: C`
- `kCFStringNormalizationFormKC :: KC`

#### `CFStringInlineBuffer` (struct)

- `buffer:              [64] u16`
- `theString:           CFStringRef`
- `directUniCharBuffer: *u16`
- `directCStringBuffer: *u8`
- `rangeToBuffer:       CFRange`
- `bufferedRangeStart:  CFIndex`
- `bufferedRangeEnd:    CFIndex`

#### `CFTimeZoneNameStyle` (enum)

- `Standard            :: 0`
- `ShortStandard       :: 1`
- `DaylightSaving      :: 2`
- `ShortDaylightSaving :: 3`
- `Generic             :: 4`
- `ShortGeneric        :: 5`
- `kCFTimeZoneNameStyleStandard            :: Standard`
- `kCFTimeZoneNameStyleShortStandard       :: ShortStandard`
- `kCFTimeZoneNameStyleDaylightSaving      :: DaylightSaving`
- `kCFTimeZoneNameStyleShortDaylightSaving :: ShortDaylightSaving`
- `kCFTimeZoneNameStyleGeneric             :: Generic`
- `kCFTimeZoneNameStyleShortGeneric        :: ShortGeneric`

#### `CFCalendarUnit` (enum)

- `Era               :: 2`
- `Year              :: 4`
- `Month             :: 8`
- `Day               :: 16`
- `Hour              :: 32`
- `Minute            :: 64`
- `Second            :: 128`
- `Week              :: 256`
- `Weekday           :: 512`
- `WeekdayOrdinal    :: 1024`
- `Quarter           :: 2048`
- `WeekOfMonth       :: 4096`
- `WeekOfYear        :: 8192`
- `YearForWeekOfYear :: 16384`
- `kCFCalendarUnitEra               :: Era`
- `kCFCalendarUnitYear              :: Year`
- `kCFCalendarUnitMonth             :: Month`
- `kCFCalendarUnitDay               :: Day`
- `kCFCalendarUnitHour              :: Hour`
- `kCFCalendarUnitMinute            :: Minute`
- `kCFCalendarUnitSecond            :: Second`
- `kCFCalendarUnitWeek              :: Week`
- `kCFCalendarUnitWeekday           :: Weekday`
- `kCFCalendarUnitWeekdayOrdinal    :: WeekdayOrdinal`
- `kCFCalendarUnitQuarter           :: Quarter`
- `kCFCalendarUnitWeekOfMonth       :: WeekOfMonth`
- `kCFCalendarUnitWeekOfYear        :: WeekOfYear`
- `kCFCalendarUnitYearForWeekOfYear :: YearForWeekOfYear`

#### `anon_enum_6` (enum)

- `kCFCalendarComponentsWrap :: 1`

#### `kCFDateFormatter` (enum)

- `NoStyle     :: 0`
- `ShortStyle  :: 1`
- `MediumStyle :: 2`
- `LongStyle   :: 3`
- `FullStyle   :: 4`
- `kCFDateFormatterNoStyle     :: NoStyle`
- `kCFDateFormatterShortStyle  :: ShortStyle`
- `kCFDateFormatterMediumStyle :: MediumStyle`
- `kCFDateFormatterLongStyle   :: LongStyle`
- `kCFDateFormatterFullStyle   :: FullStyle`

#### `kCFISO8601DateFormatWith` (enum)

- `Year                     :: 1`
- `Month                    :: 2`
- `WeekOfYear               :: 4`
- `Day                      :: 16`
- `Time                     :: 32`
- `TimeZone                 :: 64`
- `SpaceBetweenDateAndTime  :: 128`
- `DashSeparatorInDate      :: 256`
- `ColonSeparatorInTime     :: 512`
- `ColonSeparatorInTimeZone :: 1024`
- `FractionalSeconds        :: 2048`
- `FullDate                 :: 275`
- `FullTime                 :: 1632`
- `InternetDateTime         :: 1907`
- `kCFISO8601DateFormatWithYear                     :: Year`
- `kCFISO8601DateFormatWithMonth                    :: Month`
- `kCFISO8601DateFormatWithWeekOfYear               :: WeekOfYear`
- `kCFISO8601DateFormatWithDay                      :: Day`
- `kCFISO8601DateFormatWithTime                     :: Time`
- `kCFISO8601DateFormatWithTimeZone                 :: TimeZone`
- `kCFISO8601DateFormatWithSpaceBetweenDateAndTime  :: SpaceBetweenDateAndTime`
- `kCFISO8601DateFormatWithDashSeparatorInDate      :: DashSeparatorInDate`
- `kCFISO8601DateFormatWithColonSeparatorInTime     :: ColonSeparatorInTime`
- `kCFISO8601DateFormatWithColonSeparatorInTimeZone :: ColonSeparatorInTimeZone`
- `kCFISO8601DateFormatWithFractionalSeconds        :: FractionalSeconds`
- `kCFISO8601DateFormatWithFullDate                 :: FullDate`
- `kCFISO8601DateFormatWithFullTime                 :: FullTime`
- `kCFISO8601DateFormatWithInternetDateTime         :: InternetDateTime`

#### `kCFNumber` (enum)

- `SInt8Type     :: 1`
- `SInt16Type    :: 2`
- `SInt32Type    :: 3`
- `SInt64Type    :: 4`
- `Float32Type   :: 5`
- `Float64Type   :: 6`
- `CharType      :: 7`
- `ShortType     :: 8`
- `IntType       :: 9`
- `LongType      :: 10`
- `LongLongType  :: 11`
- `FloatType     :: 12`
- `DoubleType    :: 13`
- `CFIndexType   :: 14`
- `NSIntegerType :: 15`
- `CGFloatType   :: 16`
- `MaxType       :: 16`
- `kCFNumberSInt8Type     :: SInt8Type`
- `kCFNumberSInt16Type    :: SInt16Type`
- `kCFNumberSInt32Type    :: SInt32Type`
- `kCFNumberSInt64Type    :: SInt64Type`
- `kCFNumberFloat32Type   :: Float32Type`
- `kCFNumberFloat64Type   :: Float64Type`
- `kCFNumberCharType      :: CharType`
- `kCFNumberShortType     :: ShortType`
- `kCFNumberIntType       :: IntType`
- `kCFNumberLongType      :: LongType`
- `kCFNumberLongLongType  :: LongLongType`
- `kCFNumberFloatType     :: FloatType`
- `kCFNumberDoubleType    :: DoubleType`
- `kCFNumberCFIndexType   :: CFIndexType`
- `kCFNumberNSIntegerType :: NSIntegerType`
- `kCFNumberCGFloatType   :: CGFloatType`
- `kCFNumberMaxType       :: MaxType`

#### `kCFNumberFormatter` (enum)

- `NoStyle                 :: 0`
- `DecimalStyle            :: 1`
- `CurrencyStyle           :: 2`
- `PercentStyle            :: 3`
- `ScientificStyle         :: 4`
- `SpellOutStyle           :: 5`
- `OrdinalStyle            :: 6`
- `CurrencyISOCodeStyle    :: 8`
- `CurrencyPluralStyle     :: 9`
- `CurrencyAccountingStyle :: 10`
- `kCFNumberFormatterNoStyle                 :: NoStyle`
- `kCFNumberFormatterDecimalStyle            :: DecimalStyle`
- `kCFNumberFormatterCurrencyStyle           :: CurrencyStyle`
- `kCFNumberFormatterPercentStyle            :: PercentStyle`
- `kCFNumberFormatterScientificStyle         :: ScientificStyle`
- `kCFNumberFormatterSpellOutStyle           :: SpellOutStyle`
- `kCFNumberFormatterOrdinalStyle            :: OrdinalStyle`
- `kCFNumberFormatterCurrencyISOCodeStyle    :: CurrencyISOCodeStyle`
- `kCFNumberFormatterCurrencyPluralStyle     :: CurrencyPluralStyle`
- `kCFNumberFormatterCurrencyAccountingStyle :: CurrencyAccountingStyle`

#### `anon_enum_7` (enum)

- `kCFNumberFormatterParseIntegersOnly :: 1`

#### `kCFNumberFormatterRound` (enum)

- `Ceiling  :: 0`
- `Floor    :: 1`
- `Down     :: 2`
- `Up       :: 3`
- `HalfEven :: 4`
- `HalfDown :: 5`
- `HalfUp   :: 6`
- `kCFNumberFormatterRoundCeiling  :: Ceiling`
- `kCFNumberFormatterRoundFloor    :: Floor`
- `kCFNumberFormatterRoundDown     :: Down`
- `kCFNumberFormatterRoundUp       :: Up`
- `kCFNumberFormatterRoundHalfEven :: HalfEven`
- `kCFNumberFormatterRoundHalfDown :: HalfDown`
- `kCFNumberFormatterRoundHalfUp   :: HalfUp`

#### `kCFNumberFormatterPad` (enum)

- `BeforePrefix :: 0`
- `AfterPrefix  :: 1`
- `BeforeSuffix :: 2`
- `AfterSuffix  :: 3`
- `kCFNumberFormatterPadBeforePrefix :: BeforePrefix`
- `kCFNumberFormatterPadAfterPrefix  :: AfterPrefix`
- `kCFNumberFormatterPadBeforeSuffix :: BeforeSuffix`
- `kCFNumberFormatterPadAfterSuffix  :: AfterSuffix`

#### `kCFURL` (enum)

- `POSIXPathStyle   :: 0`
- `HFSPathStyle     :: 1`
- `WindowsPathStyle :: 2`
- `kCFURLPOSIXPathStyle   :: POSIXPathStyle`
- `kCFURLHFSPathStyle     :: HFSPathStyle`
- `kCFURLWindowsPathStyle :: WindowsPathStyle`

#### `kCFURLComponent` (enum)

- `Scheme            :: 1`
- `NetLocation       :: 2`
- `Path              :: 3`
- `ResourceSpecifier :: 4`
- `User              :: 5`
- `Password          :: 6`
- `UserInfo          :: 7`
- `Host              :: 8`
- `Port              :: 9`
- `ParameterString   :: 10`
- `Query             :: 11`
- `Fragment          :: 12`
- `kCFURLComponentScheme            :: Scheme`
- `kCFURLComponentNetLocation       :: NetLocation`
- `kCFURLComponentPath              :: Path`
- `kCFURLComponentResourceSpecifier :: ResourceSpecifier`
- `kCFURLComponentUser              :: User`
- `kCFURLComponentPassword          :: Password`
- `kCFURLComponentUserInfo          :: UserInfo`
- `kCFURLComponentHost              :: Host`
- `kCFURLComponentPort              :: Port`
- `kCFURLComponentParameterString   :: ParameterString`
- `kCFURLComponentQuery             :: Query`
- `kCFURLComponentFragment          :: Fragment`

#### `kCFURLBookmarkCreation` (enum)

- `MinimalBookmarkMask              :: 512`
- `SuitableForBookmarkFile          :: 1024`
- `WithSecurityScope                :: 2048`
- `SecurityScopeAllowOnlyReadAccess :: 4096`
- `PreferFileIDResolutionMask       :: 256`
- `kCFURLBookmarkCreationMinimalBookmarkMask              :: MinimalBookmarkMask`
- `kCFURLBookmarkCreationSuitableForBookmarkFile          :: SuitableForBookmarkFile`
- `kCFURLBookmarkCreationWithSecurityScope                :: WithSecurityScope`
- `kCFURLBookmarkCreationSecurityScopeAllowOnlyReadAccess :: SecurityScopeAllowOnlyReadAccess`
- `kCFURLBookmarkCreationPreferFileIDResolutionMask       :: PreferFileIDResolutionMask`

#### `kCF` (enum)

- `URLBookmarkResolutionWithoutUIMask       :: 256`
- `URLBookmarkResolutionWithoutMountingMask :: 512`
- `URLBookmarkResolutionWithSecurityScope   :: 1024`
- `BookmarkResolutionWithoutUIMask          :: 256`
- `BookmarkResolutionWithoutMountingMask    :: 512`
- `kCFURLBookmarkResolutionWithoutUIMask       :: URLBookmarkResolutionWithoutUIMask`
- `kCFURLBookmarkResolutionWithoutMountingMask :: URLBookmarkResolutionWithoutMountingMask`
- `kCFURLBookmarkResolutionWithSecurityScope   :: URLBookmarkResolutionWithSecurityScope`
- `kCFBookmarkResolutionWithoutUIMask          :: BookmarkResolutionWithoutUIMask`
- `kCFBookmarkResolutionWithoutMountingMask    :: BookmarkResolutionWithoutMountingMask`

#### `CFRunLoopRunResult` (enum)

- `Finished      :: 1`
- `Stopped       :: 2`
- `TimedOut      :: 3`
- `HandledSource :: 4`
- `kCFRunLoopRunFinished      :: Finished`
- `kCFRunLoopRunStopped       :: Stopped`
- `kCFRunLoopRunTimedOut      :: TimedOut`
- `kCFRunLoopRunHandledSource :: HandledSource`

#### `CFRunLoopActivity` (enum_flags)

- `Entry         :: 0x1`
- `BeforeTimers  :: 0x2`
- `BeforeSources :: 0x4`
- `BeforeWaiting :: 0x20`
- `AfterWaiting  :: 0x40`
- `Exit          :: 0x80`
- `AllActivities :: 0xfffffff`
- `kCFRunLoopEntry         :: Entry`
- `kCFRunLoopBeforeTimers  :: BeforeTimers`
- `kCFRunLoopBeforeSources :: BeforeSources`
- `kCFRunLoopBeforeWaiting :: BeforeWaiting`
- `kCFRunLoopAfterWaiting  :: AfterWaiting`
- `kCFRunLoopExit          :: Exit`
- `kCFRunLoopAllActivities :: AllActivities`

#### `CFRunLoopSourceContext` (struct)

- `version:         CFIndex`
- `info:            *void`
- `retain:          #type (info: *void) -> *void #c_call`
- `release:         #type (info: *void) -> void #c_call`
- `copyDescription: #type (info: *void) -> CFStringRef #c_call`
- `equal:           #type (info1: *void, info2: *void) -> Boolean #c_call`
- `hash:            #type (info: *void) -> CFHashCode #c_call`
- `schedule:        #type (info: *void, rl: CFRunLoopRef, mode: CFRunLoopMode) -> void #c_call`
- `cancel:          #type (info: *void, rl: CFRunLoopRef, mode: CFRunLoopMode) -> void #c_call`
- `perform:         #type (info: *void) -> void #c_call`

#### `CFRunLoopSourceContext1` (struct)

- `version:         CFIndex`
- `info:            *void`
- `retain:          #type (info: *void) -> *void #c_call`
- `release:         #type (info: *void) -> void #c_call`
- `copyDescription: #type (info: *void) -> CFStringRef #c_call`
- `equal:           #type (info1: *void, info2: *void) -> Boolean #c_call`
- `hash:            #type (info: *void) -> CFHashCode #c_call`
- `getPort:         #type (info: *void) -> mach_port_t #c_call`
- `perform:         #type (msg: *void, size: CFIndex, allocator: CFAllocatorRef, info: *void) -> *void #c_call`

#### `CFRunLoopObserverContext` (struct)

- `version:         CFIndex`
- `info:            *void`
- `retain:          #type (info: *void) -> *void #c_call`
- `release:         #type (info: *void) -> void #c_call`
- `copyDescription: #type (info: *void) -> CFStringRef #c_call`

#### `CFRunLoopTimerContext` (struct)

- `version:         CFIndex`
- `info:            *void`
- `retain:          #type (info: *void) -> *void #c_call`
- `release:         #type (info: *void) -> void #c_call`
- `copyDescription: #type (info: *void) -> CFStringRef #c_call`

#### `kCFSocket` (enum)

- `Success :: 0`
- `Error   :: -1`
- `Timeout :: -2`
- `kCFSocketSuccess :: Success`
- `kCFSocketError   :: Error`
- `kCFSocketTimeout :: Timeout`

#### `CFSocketSignature` (struct)

- `protocolFamily: s32`
- `socketType:     s32`
- `protocol:       s32`
- `address:        CFDataRef`

#### `kCFSocket_1` (enum)

- `NoCallBack      :: 0`
- `ReadCallBack    :: 1`
- `AcceptCallBack  :: 2`
- `DataCallBack    :: 3`
- `ConnectCallBack :: 4`
- `WriteCallBack   :: 8`
- `kCFSocketNoCallBack      :: NoCallBack`
- `kCFSocketReadCallBack    :: ReadCallBack`
- `kCFSocketAcceptCallBack  :: AcceptCallBack`
- `kCFSocketDataCallBack    :: DataCallBack`
- `kCFSocketConnectCallBack :: ConnectCallBack`
- `kCFSocketWriteCallBack   :: WriteCallBack`

#### `kCFSocket_2` (enum)

- `AutomaticallyReenableReadCallBack   :: 1`
- `AutomaticallyReenableAcceptCallBack :: 2`
- `AutomaticallyReenableDataCallBack   :: 3`
- `AutomaticallyReenableWriteCallBack  :: 8`
- `LeaveErrors                         :: 64`
- `CloseOnInvalidate                   :: 128`
- `kCFSocketAutomaticallyReenableReadCallBack   :: AutomaticallyReenableReadCallBack`
- `kCFSocketAutomaticallyReenableAcceptCallBack :: AutomaticallyReenableAcceptCallBack`
- `kCFSocketAutomaticallyReenableDataCallBack   :: AutomaticallyReenableDataCallBack`
- `kCFSocketAutomaticallyReenableWriteCallBack  :: AutomaticallyReenableWriteCallBack`
- `kCFSocketLeaveErrors                         :: LeaveErrors`
- `kCFSocketCloseOnInvalidate                   :: CloseOnInvalidate`

#### `CFSocketContext` (struct)

- `version:         CFIndex`
- `info:            *void`
- `retain:          #type (info: *void) -> *void #c_call`
- `release:         #type (info: *void) -> void #c_call`
- `copyDescription: #type (info: *void) -> CFStringRef #c_call`

#### `CFStreamStatus` (enum)

- `NotOpen :: 0`
- `Opening :: 1`
- `Open    :: 2`
- `Reading :: 3`
- `Writing :: 4`
- `AtEnd   :: 5`
- `Closed  :: 6`
- `Error   :: 7`
- `kCFStreamStatusNotOpen :: NotOpen`
- `kCFStreamStatusOpening :: Opening`
- `kCFStreamStatusOpen    :: Open`
- `kCFStreamStatusReading :: Reading`
- `kCFStreamStatusWriting :: Writing`
- `kCFStreamStatusAtEnd   :: AtEnd`
- `kCFStreamStatusClosed  :: Closed`
- `kCFStreamStatusError   :: Error`

#### `kCFStreamEvent` (enum)

- `None              :: 0`
- `OpenCompleted     :: 1`
- `HasBytesAvailable :: 2`
- `CanAcceptBytes    :: 4`
- `ErrorOccurred     :: 8`
- `EndEncountered    :: 16`
- `kCFStreamEventNone              :: None`
- `kCFStreamEventOpenCompleted     :: OpenCompleted`
- `kCFStreamEventHasBytesAvailable :: HasBytesAvailable`
- `kCFStreamEventCanAcceptBytes    :: CanAcceptBytes`
- `kCFStreamEventErrorOccurred     :: ErrorOccurred`
- `kCFStreamEventEndEncountered    :: EndEncountered`

#### `CFStreamClientContext` (struct)

- `version:         CFIndex`
- `info:            *void`
- `retain:          #type (info: *void) -> *void #c_call`
- `release:         #type (info: *void) -> void #c_call`
- `copyDescription: #type (info: *void) -> CFStringRef #c_call`

#### `CFStreamErrorDomain` (enum)

- `Custom      :: -1`
- `POSIX       :: 1`
- `MacOSStatus :: 2`
- `kCFStreamErrorDomainCustom      :: Custom`
- `kCFStreamErrorDomainPOSIX       :: POSIX`
- `kCFStreamErrorDomainMacOSStatus :: MacOSStatus`

#### `CFStreamError` (struct)

- `domain: CFIndex`
- `error:  s32`

#### `kCFPropertyList` (enum)

- `Immutable                  :: 0`
- `MutableContainers          :: 1`
- `MutableContainersAndLeaves :: 2`
- `kCFPropertyListImmutable                  :: Immutable`
- `kCFPropertyListMutableContainers          :: MutableContainers`
- `kCFPropertyListMutableContainersAndLeaves :: MutableContainersAndLeaves`

#### `kCFPropertyList_1` (enum)

- `OpenStepFormat    :: 1`
- `XMLFormat_v1_0    :: 100`
- `BinaryFormat_v1_0 :: 200`
- `kCFPropertyListOpenStepFormat    :: OpenStepFormat`
- `kCFPropertyListXMLFormat_v1_0    :: XMLFormat_v1_0`
- `kCFPropertyListBinaryFormat_v1_0 :: BinaryFormat_v1_0`

#### `kCFPropertyList_2` (enum)

- `ReadCorruptError        :: 3840`
- `ReadUnknownVersionError :: 3841`
- `ReadStreamError         :: 3842`
- `WriteStreamError        :: 3851`
- `kCFPropertyListReadCorruptError        :: ReadCorruptError`
- `kCFPropertyListReadUnknownVersionError :: ReadUnknownVersionError`
- `kCFPropertyListReadStreamError         :: ReadStreamError`
- `kCFPropertyListWriteStreamError        :: WriteStreamError`

#### `kCFStringEncoding_1` (enum)

- `MacJapanese             :: 1`
- `MacChineseTrad          :: 2`
- `MacKorean               :: 3`
- `MacArabic               :: 4`
- `MacHebrew               :: 5`
- `MacGreek                :: 6`
- `MacCyrillic             :: 7`
- `MacDevanagari           :: 9`
- `MacGurmukhi             :: 10`
- `MacGujarati             :: 11`
- `MacOriya                :: 12`
- `MacBengali              :: 13`
- `MacTamil                :: 14`
- `MacTelugu               :: 15`
- `MacKannada              :: 16`
- `MacMalayalam            :: 17`
- `MacSinhalese            :: 18`
- `MacBurmese              :: 19`
- `MacKhmer                :: 20`
- `MacThai                 :: 21`
- `MacLaotian              :: 22`
- `MacGeorgian             :: 23`
- `MacArmenian             :: 24`
- `MacChineseSimp          :: 25`
- `MacTibetan              :: 26`
- `MacMongolian            :: 27`
- `MacEthiopic             :: 28`
- `MacCentralEurRoman      :: 29`
- `MacVietnamese           :: 30`
- `MacExtArabic            :: 31`
- `MacSymbol               :: 33`
- `MacDingbats             :: 34`
- `MacTurkish              :: 35`
- `MacCroatian             :: 36`
- `MacIcelandic            :: 37`
- `MacRomanian             :: 38`
- `MacCeltic               :: 39`
- `MacGaelic               :: 40`
- `MacFarsi                :: 140`
- `MacUkrainian            :: 152`
- `MacInuit                :: 236`
- `MacVT100                :: 252`
- `MacHFS                  :: 255`
- `ISOLatin2               :: 514`
- `ISOLatin3               :: 515`
- `ISOLatin4               :: 516`
- `ISOLatinCyrillic        :: 517`
- `ISOLatinArabic          :: 518`
- `ISOLatinGreek           :: 519`
- `ISOLatinHebrew          :: 520`
- `ISOLatin5               :: 521`
- `ISOLatin6               :: 522`
- `ISOLatinThai            :: 523`
- `ISOLatin7               :: 525`
- `ISOLatin8               :: 526`
- `ISOLatin9               :: 527`
- `ISOLatin10              :: 528`
- `DOSLatinUS              :: 1024`
- `DOSGreek                :: 1029`
- `DOSBalticRim            :: 1030`
- `DOSLatin1               :: 1040`
- `DOSGreek1               :: 1041`
- `DOSLatin2               :: 1042`
- `DOSCyrillic             :: 1043`
- `DOSTurkish              :: 1044`
- `DOSPortuguese           :: 1045`
- `DOSIcelandic            :: 1046`
- `DOSHebrew               :: 1047`
- `DOSCanadianFrench       :: 1048`
- `DOSArabic               :: 1049`
- `DOSNordic               :: 1050`
- `DOSRussian              :: 1051`
- `DOSGreek2               :: 1052`
- `DOSThai                 :: 1053`
- `DOSJapanese             :: 1056`
- `DOSChineseSimplif       :: 1057`
- `DOSKorean               :: 1058`
- `DOSChineseTrad          :: 1059`
- `WindowsLatin2           :: 1281`
- `WindowsCyrillic         :: 1282`
- `WindowsGreek            :: 1283`
- `WindowsLatin5           :: 1284`
- `WindowsHebrew           :: 1285`
- `WindowsArabic           :: 1286`
- `WindowsBalticRim        :: 1287`
- `WindowsVietnamese       :: 1288`
- `WindowsKoreanJohab      :: 1296`
- `ANSEL                   :: 1537`
- `JIS_X0201_76            :: 1568`
- `JIS_X0208_83            :: 1569`
- `JIS_X0208_90            :: 1570`
- `JIS_X0212_90            :: 1571`
- `JIS_C6226_78            :: 1572`
- `ShiftJIS_X0213          :: 1576`
- `ShiftJIS_X0213_MenKuTen :: 1577`
- `GB_2312_80              :: 1584`
- `GBK_95                  :: 1585`
- `GB_18030_2000           :: 1586`
- `KSC_5601_87             :: 1600`
- `KSC_5601_92_Johab       :: 1601`
- `CNS_11643_92_P1         :: 1617`
- `CNS_11643_92_P2         :: 1618`
- `CNS_11643_92_P3         :: 1619`
- `ISO_2022_JP             :: 2080`
- `ISO_2022_JP_2           :: 2081`
- `ISO_2022_JP_1           :: 2082`
- `ISO_2022_JP_3           :: 2083`
- `ISO_2022_CN             :: 2096`
- `ISO_2022_CN_EXT         :: 2097`
- `ISO_2022_KR             :: 2112`
- `EUC_JP                  :: 2336`
- `EUC_CN                  :: 2352`
- `EUC_TW                  :: 2353`
- `EUC_KR                  :: 2368`
- `ShiftJIS                :: 2561`
- `KOI8_R                  :: 2562`
- `Big5                    :: 2563`
- `MacRomanLatin1          :: 2564`
- `HZ_GB_2312              :: 2565`
- `Big5_HKSCS_1999         :: 2566`
- `VISCII                  :: 2567`
- `KOI8_U                  :: 2568`
- `Big5_E                  :: 2569`
- `NextStepJapanese        :: 2818`
- `EBCDIC_US               :: 3073`
- `EBCDIC_CP037            :: 3074`
- `UTF7                    :: 67109120`
- `UTF7_IMAP               :: 2576`
- `ShiftJIS_X0213_00       :: 1576`
- `kCFStringEncodingMacJapanese             :: MacJapanese`
- `kCFStringEncodingMacChineseTrad          :: MacChineseTrad`
- `kCFStringEncodingMacKorean               :: MacKorean`
- `kCFStringEncodingMacArabic               :: MacArabic`
- `kCFStringEncodingMacHebrew               :: MacHebrew`
- `kCFStringEncodingMacGreek                :: MacGreek`
- `kCFStringEncodingMacCyrillic             :: MacCyrillic`
- `kCFStringEncodingMacDevanagari           :: MacDevanagari`
- `kCFStringEncodingMacGurmukhi             :: MacGurmukhi`
- `kCFStringEncodingMacGujarati             :: MacGujarati`
- `kCFStringEncodingMacOriya                :: MacOriya`
- `kCFStringEncodingMacBengali              :: MacBengali`
- `kCFStringEncodingMacTamil                :: MacTamil`
- `kCFStringEncodingMacTelugu               :: MacTelugu`
- `kCFStringEncodingMacKannada              :: MacKannada`
- `kCFStringEncodingMacMalayalam            :: MacMalayalam`
- `kCFStringEncodingMacSinhalese            :: MacSinhalese`
- `kCFStringEncodingMacBurmese              :: MacBurmese`
- `kCFStringEncodingMacKhmer                :: MacKhmer`
- `kCFStringEncodingMacThai                 :: MacThai`
- `kCFStringEncodingMacLaotian              :: MacLaotian`
- `kCFStringEncodingMacGeorgian             :: MacGeorgian`
- `kCFStringEncodingMacArmenian             :: MacArmenian`
- `kCFStringEncodingMacChineseSimp          :: MacChineseSimp`
- `kCFStringEncodingMacTibetan              :: MacTibetan`
- `kCFStringEncodingMacMongolian            :: MacMongolian`
- `kCFStringEncodingMacEthiopic             :: MacEthiopic`
- `kCFStringEncodingMacCentralEurRoman      :: MacCentralEurRoman`
- `kCFStringEncodingMacVietnamese           :: MacVietnamese`
- `kCFStringEncodingMacExtArabic            :: MacExtArabic`
- `kCFStringEncodingMacSymbol               :: MacSymbol`
- `kCFStringEncodingMacDingbats             :: MacDingbats`
- `kCFStringEncodingMacTurkish              :: MacTurkish`
- `kCFStringEncodingMacCroatian             :: MacCroatian`
- `kCFStringEncodingMacIcelandic            :: MacIcelandic`
- `kCFStringEncodingMacRomanian             :: MacRomanian`
- `kCFStringEncodingMacCeltic               :: MacCeltic`
- `kCFStringEncodingMacGaelic               :: MacGaelic`
- `kCFStringEncodingMacFarsi                :: MacFarsi`
- `kCFStringEncodingMacUkrainian            :: MacUkrainian`
- `kCFStringEncodingMacInuit                :: MacInuit`
- `kCFStringEncodingMacVT100                :: MacVT100`
- `kCFStringEncodingMacHFS                  :: MacHFS`
- `kCFStringEncodingISOLatin2               :: ISOLatin2`
- `kCFStringEncodingISOLatin3               :: ISOLatin3`
- `kCFStringEncodingISOLatin4               :: ISOLatin4`
- `kCFStringEncodingISOLatinCyrillic        :: ISOLatinCyrillic`
- `kCFStringEncodingISOLatinArabic          :: ISOLatinArabic`
- `kCFStringEncodingISOLatinGreek           :: ISOLatinGreek`
- `kCFStringEncodingISOLatinHebrew          :: ISOLatinHebrew`
- `kCFStringEncodingISOLatin5               :: ISOLatin5`
- `kCFStringEncodingISOLatin6               :: ISOLatin6`
- `kCFStringEncodingISOLatinThai            :: ISOLatinThai`
- `kCFStringEncodingISOLatin7               :: ISOLatin7`
- `kCFStringEncodingISOLatin8               :: ISOLatin8`
- `kCFStringEncodingISOLatin9               :: ISOLatin9`
- `kCFStringEncodingISOLatin10              :: ISOLatin10`
- `kCFStringEncodingDOSLatinUS              :: DOSLatinUS`
- `kCFStringEncodingDOSGreek                :: DOSGreek`
- `kCFStringEncodingDOSBalticRim            :: DOSBalticRim`
- `kCFStringEncodingDOSLatin1               :: DOSLatin1`
- `kCFStringEncodingDOSGreek1               :: DOSGreek1`
- `kCFStringEncodingDOSLatin2               :: DOSLatin2`
- `kCFStringEncodingDOSCyrillic             :: DOSCyrillic`
- `kCFStringEncodingDOSTurkish              :: DOSTurkish`
- `kCFStringEncodingDOSPortuguese           :: DOSPortuguese`
- `kCFStringEncodingDOSIcelandic            :: DOSIcelandic`
- `kCFStringEncodingDOSHebrew               :: DOSHebrew`
- `kCFStringEncodingDOSCanadianFrench       :: DOSCanadianFrench`
- `kCFStringEncodingDOSArabic               :: DOSArabic`
- `kCFStringEncodingDOSNordic               :: DOSNordic`
- `kCFStringEncodingDOSRussian              :: DOSRussian`
- `kCFStringEncodingDOSGreek2               :: DOSGreek2`
- `kCFStringEncodingDOSThai                 :: DOSThai`
- `kCFStringEncodingDOSJapanese             :: DOSJapanese`
- `kCFStringEncodingDOSChineseSimplif       :: DOSChineseSimplif`
- `kCFStringEncodingDOSKorean               :: DOSKorean`
- `kCFStringEncodingDOSChineseTrad          :: DOSChineseTrad`
- `kCFStringEncodingWindowsLatin2           :: WindowsLatin2`
- `kCFStringEncodingWindowsCyrillic         :: WindowsCyrillic`
- `kCFStringEncodingWindowsGreek            :: WindowsGreek`
- `kCFStringEncodingWindowsLatin5           :: WindowsLatin5`
- `kCFStringEncodingWindowsHebrew           :: WindowsHebrew`
- `kCFStringEncodingWindowsArabic           :: WindowsArabic`
- `kCFStringEncodingWindowsBalticRim        :: WindowsBalticRim`
- `kCFStringEncodingWindowsVietnamese       :: WindowsVietnamese`
- `kCFStringEncodingWindowsKoreanJohab      :: WindowsKoreanJohab`
- `kCFStringEncodingANSEL                   :: ANSEL`
- `kCFStringEncodingJIS_X0201_76            :: JIS_X0201_76`
- `kCFStringEncodingJIS_X0208_83            :: JIS_X0208_83`
- `kCFStringEncodingJIS_X0208_90            :: JIS_X0208_90`
- `kCFStringEncodingJIS_X0212_90            :: JIS_X0212_90`
- `kCFStringEncodingJIS_C6226_78            :: JIS_C6226_78`
- `kCFStringEncodingShiftJIS_X0213          :: ShiftJIS_X0213`
- `kCFStringEncodingShiftJIS_X0213_MenKuTen :: ShiftJIS_X0213_MenKuTen`
- `kCFStringEncodingGB_2312_80              :: GB_2312_80`
- `kCFStringEncodingGBK_95                  :: GBK_95`
- `kCFStringEncodingGB_18030_2000           :: GB_18030_2000`
- `kCFStringEncodingKSC_5601_87             :: KSC_5601_87`
- `kCFStringEncodingKSC_5601_92_Johab       :: KSC_5601_92_Johab`
- `kCFStringEncodingCNS_11643_92_P1         :: CNS_11643_92_P1`
- `kCFStringEncodingCNS_11643_92_P2         :: CNS_11643_92_P2`
- `kCFStringEncodingCNS_11643_92_P3         :: CNS_11643_92_P3`
- `kCFStringEncodingISO_2022_JP             :: ISO_2022_JP`
- `kCFStringEncodingISO_2022_JP_2           :: ISO_2022_JP_2`
- `kCFStringEncodingISO_2022_JP_1           :: ISO_2022_JP_1`
- `kCFStringEncodingISO_2022_JP_3           :: ISO_2022_JP_3`
- `kCFStringEncodingISO_2022_CN             :: ISO_2022_CN`
- `kCFStringEncodingISO_2022_CN_EXT         :: ISO_2022_CN_EXT`
- `kCFStringEncodingISO_2022_KR             :: ISO_2022_KR`
- `kCFStringEncodingEUC_JP                  :: EUC_JP`
- `kCFStringEncodingEUC_CN                  :: EUC_CN`
- `kCFStringEncodingEUC_TW                  :: EUC_TW`
- `kCFStringEncodingEUC_KR                  :: EUC_KR`
- `kCFStringEncodingShiftJIS                :: ShiftJIS`
- `kCFStringEncodingKOI8_R                  :: KOI8_R`
- `kCFStringEncodingBig5                    :: Big5`
- `kCFStringEncodingMacRomanLatin1          :: MacRomanLatin1`
- `kCFStringEncodingHZ_GB_2312              :: HZ_GB_2312`
- `kCFStringEncodingBig5_HKSCS_1999         :: Big5_HKSCS_1999`
- `kCFStringEncodingVISCII                  :: VISCII`
- `kCFStringEncodingKOI8_U                  :: KOI8_U`
- `kCFStringEncodingBig5_E                  :: Big5_E`
- `kCFStringEncodingNextStepJapanese        :: NextStepJapanese`
- `kCFStringEncodingEBCDIC_US               :: EBCDIC_US`
- `kCFStringEncodingEBCDIC_CP037            :: EBCDIC_CP037`
- `kCFStringEncodingUTF7                    :: UTF7`
- `kCFStringEncodingUTF7_IMAP               :: UTF7_IMAP`
- `kCFStringEncodingShiftJIS_X0213_00       :: ShiftJIS_X0213_00`

#### `kCFURL_1` (enum)

- `UnknownError                 :: -10`
- `UnknownSchemeError           :: -11`
- `ResourceNotFoundError        :: -12`
- `ResourceAccessViolationError :: -13`
- `RemoteHostUnavailableError   :: -14`
- `ImproperArgumentsError       :: -15`
- `UnknownPropertyKeyError      :: -16`
- `PropertyKeyUnavailableError  :: -17`
- `TimeoutError                 :: -18`
- `kCFURLUnknownError                 :: UnknownError`
- `kCFURLUnknownSchemeError           :: UnknownSchemeError`
- `kCFURLResourceNotFoundError        :: ResourceNotFoundError`
- `kCFURLResourceAccessViolationError :: ResourceAccessViolationError`
- `kCFURLRemoteHostUnavailableError   :: RemoteHostUnavailableError`
- `kCFURLImproperArgumentsError       :: ImproperArgumentsError`
- `kCFURLUnknownPropertyKeyError      :: UnknownPropertyKeyError`
- `kCFURLPropertyKeyUnavailableError  :: PropertyKeyUnavailableError`
- `kCFURLTimeoutError                 :: TimeoutError`

#### `CFUUIDBytes` (struct)

- `byte0:  u8`
- `byte1:  u8`
- `byte2:  u8`
- `byte3:  u8`
- `byte4:  u8`
- `byte5:  u8`
- `byte6:  u8`
- `byte7:  u8`
- `byte8:  u8`
- `byte9:  u8`
- `byte10: u8`
- `byte11: u8`
- `byte12: u8`
- `byte13: u8`
- `byte14: u8`
- `byte15: u8`

#### `kCFBundleExecutableArchitecture` (enum)

- `I386   :: 7`
- `PPC    :: 18`
- `X86_64 :: 16777223`
- `PPC64  :: 16777234`
- `kCFBundleExecutableArchitectureI386   :: I386`
- `kCFBundleExecutableArchitecturePPC    :: PPC`
- `kCFBundleExecutableArchitectureX86_64 :: X86_64`
- `kCFBundleExecutableArchitecturePPC64  :: PPC64`

#### `kCFMessagePort` (enum)

- `Success            :: 0`
- `SendTimeout        :: -1`
- `ReceiveTimeout     :: -2`
- `IsInvalid          :: -3`
- `TransportError     :: -4`
- `BecameInvalidError :: -5`
- `kCFMessagePortSuccess            :: Success`
- `kCFMessagePortSendTimeout        :: SendTimeout`
- `kCFMessagePortReceiveTimeout     :: ReceiveTimeout`
- `kCFMessagePortIsInvalid          :: IsInvalid`
- `kCFMessagePortTransportError     :: TransportError`
- `kCFMessagePortBecameInvalidError :: BecameInvalidError`

#### `CFMessagePortContext` (struct)

- `version:         CFIndex`
- `info:            *void`
- `retain:          #type (info: *void) -> *void #c_call`
- `release:         #type (info: *void) -> void #c_call`
- `copyDescription: #type (info: *void) -> CFStringRef #c_call`

#### `CFMachPortContext` (struct)

- `version:         CFIndex`
- `info:            *void`
- `retain:          #type (info: *void) -> *void #c_call`
- `release:         #type (info: *void) -> void #c_call`
- `copyDescription: #type (info: *void) -> CFStringRef #c_call`

#### `kCFURLEnumerator` (enum)

- `DefaultBehavior             :: 0`
- `DescendRecursively          :: 1`
- `SkipInvisibles              :: 2`
- `GenerateFileReferenceURLs   :: 4`
- `SkipPackageContents         :: 8`
- `IncludeDirectoriesPreOrder  :: 16`
- `IncludeDirectoriesPostOrder :: 32`
- `kCFURLEnumeratorDefaultBehavior             :: DefaultBehavior`
- `kCFURLEnumeratorDescendRecursively          :: DescendRecursively`
- `kCFURLEnumeratorSkipInvisibles              :: SkipInvisibles`
- `kCFURLEnumeratorGenerateFileReferenceURLs   :: GenerateFileReferenceURLs`
- `kCFURLEnumeratorSkipPackageContents         :: SkipPackageContents`
- `kCFURLEnumeratorIncludeDirectoriesPreOrder  :: IncludeDirectoriesPreOrder`
- `kCFURLEnumeratorIncludeDirectoriesPostOrder :: IncludeDirectoriesPostOrder`

#### `kCFURLEnumerator_1` (enum)

- `Success                   :: 1`
- `End                       :: 2`
- `Error                     :: 3`
- `DirectoryPostOrderSuccess :: 4`
- `kCFURLEnumeratorSuccess                   :: Success`
- `kCFURLEnumeratorEnd                       :: End`
- `kCFURLEnumeratorError                     :: Error`
- `kCFURLEnumeratorDirectoryPostOrderSuccess :: DirectoryPostOrderSuccess`

#### `kCFStringTokenizer` (enum)

- `UnitWord                    :: 0`
- `UnitSentence                :: 1`
- `UnitParagraph               :: 2`
- `UnitLineBreak               :: 3`
- `UnitWordBoundary            :: 4`
- `AttributeLatinTranscription :: 65536`
- `AttributeLanguage           :: 131072`
- `kCFStringTokenizerUnitWord                    :: UnitWord`
- `kCFStringTokenizerUnitSentence                :: UnitSentence`
- `kCFStringTokenizerUnitParagraph               :: UnitParagraph`
- `kCFStringTokenizerUnitLineBreak               :: UnitLineBreak`
- `kCFStringTokenizerUnitWordBoundary            :: UnitWordBoundary`
- `kCFStringTokenizerAttributeLatinTranscription :: AttributeLatinTranscription`
- `kCFStringTokenizerAttributeLanguage           :: AttributeLanguage`

#### `kCFStringTokenizerToken` (enum)

- `None                    :: 0`
- `Normal                  :: 1`
- `HasSubTokensMask        :: 2`
- `HasDerivedSubTokensMask :: 4`
- `HasHasNumbersMask       :: 8`
- `HasNonLettersMask       :: 16`
- `IsCJWordMask            :: 32`
- `kCFStringTokenizerTokenNone                    :: None`
- `kCFStringTokenizerTokenNormal                  :: Normal`
- `kCFStringTokenizerTokenHasSubTokensMask        :: HasSubTokensMask`
- `kCFStringTokenizerTokenHasDerivedSubTokensMask :: HasDerivedSubTokensMask`
- `kCFStringTokenizerTokenHasHasNumbersMask       :: HasHasNumbersMask`
- `kCFStringTokenizerTokenHasNonLettersMask       :: HasNonLettersMask`
- `kCFStringTokenizerTokenIsCJWordMask            :: IsCJWordMask`

#### `kCFFileDescriptor` (enum)

- `ReadCallBack  :: 1`
- `WriteCallBack :: 2`
- `kCFFileDescriptorReadCallBack  :: ReadCallBack`
- `kCFFileDescriptorWriteCallBack :: WriteCallBack`

#### `CFFileDescriptorContext` (struct)

- `version:         CFIndex`
- `info:            *void`
- `retain:          #type (info: *void) -> *void #c_call`
- `release:         #type (info: *void) -> void #c_call`
- `copyDescription: #type (info: *void) -> CFStringRef #c_call`

#### `kCFUserNotification` (enum)

- `StopAlertLevel    :: 0`
- `NoteAlertLevel    :: 1`
- `CautionAlertLevel :: 2`
- `PlainAlertLevel   :: 3`
- `kCFUserNotificationStopAlertLevel    :: StopAlertLevel`
- `kCFUserNotificationNoteAlertLevel    :: NoteAlertLevel`
- `kCFUserNotificationCautionAlertLevel :: CautionAlertLevel`
- `kCFUserNotificationPlainAlertLevel   :: PlainAlertLevel`

#### `kCFUserNotification_1` (enum)

- `DefaultResponse   :: 0`
- `AlternateResponse :: 1`
- `OtherResponse     :: 2`
- `CancelResponse    :: 3`
- `kCFUserNotificationDefaultResponse   :: DefaultResponse`
- `kCFUserNotificationAlternateResponse :: AlternateResponse`
- `kCFUserNotificationOtherResponse     :: OtherResponse`
- `kCFUserNotificationCancelResponse    :: CancelResponse`

#### `kCFUserNotification_2` (enum)

- `NoDefaultButtonFlag :: 32`
- `UseRadioButtonsFlag :: 64`
- `kCFUserNotificationNoDefaultButtonFlag :: NoDefaultButtonFlag`
- `kCFUserNotificationUseRadioButtonsFlag :: UseRadioButtonsFlag`

#### `anon_enum_8` (enum)

- `kCFXMLNodeCurrentVersion :: 1`

#### `__mbstate_t` (struct)

- `__mbstate8: [128] u8`
- `#overlay (__mbstate8) _mbstateL: s64`

#### `KERN` (enum)

- `SUCCESS                  :: 0`
- `INVALID_ADDRESS          :: 1`
- `PROTECTION_FAILURE       :: 2`
- `NO_SPACE                 :: 3`
- `INVALID_ARGUMENT         :: 4`
- `FAILURE                  :: 5`
- `RESOURCE_SHORTAGE        :: 6`
- `NOT_RECEIVER             :: 7`
- `NO_ACCESS                :: 8`
- `MEMORY_FAILURE           :: 9`
- `MEMORY_ERROR             :: 10`
- `ALREADY_IN_SET           :: 11`
- `NOT_IN_SET               :: 12`
- `NAME_EXISTS              :: 13`
- `ABORTED                  :: 14`
- `INVALID_NAME             :: 15`
- `INVALID_TASK             :: 16`
- `INVALID_RIGHT            :: 17`
- `INVALID_VALUE            :: 18`
- `UREFS_OVERFLOW           :: 19`
- `INVALID_CAPABILITY       :: 20`
- `RIGHT_EXISTS             :: 21`
- `INVALID_HOST             :: 22`
- `MEMORY_PRESENT           :: 23`
- `MEMORY_DATA_MOVED        :: 24`
- `MEMORY_RESTART_COPY      :: 25`
- `INVALID_PROCESSOR_SET    :: 26`
- `POLICY_LIMIT             :: 27`
- `INVALID_POLICY           :: 28`
- `INVALID_OBJECT           :: 29`
- `ALREADY_WAITING          :: 30`
- `DEFAULT_SET              :: 31`
- `EXCEPTION_PROTECTED      :: 32`
- `INVALID_LEDGER           :: 33`
- `INVALID_MEMORY_CONTROL   :: 34`
- `INVALID_SECURITY         :: 35`
- `NOT_DEPRESSED            :: 36`
- `TERMINATED               :: 37`
- `LOCK_SET_DESTROYED       :: 38`
- `LOCK_UNSTABLE            :: 39`
- `LOCK_OWNED               :: 40`
- `LOCK_OWNED_SELF          :: 41`
- `SEMAPHORE_DESTROYED      :: 42`
- `RPC_SERVER_TERMINATED    :: 43`
- `RPC_TERMINATE_ORPHAN     :: 44`
- `RPC_CONTINUE_ORPHAN      :: 45`
- `NOT_SUPPORTED            :: 46`
- `NODE_DOWN                :: 47`
- `NOT_WAITING              :: 48`
- `OPERATION_TIMED_OUT      :: 49`
- `CODESIGN_ERROR           :: 50`
- `POLICY_STATIC            :: 51`
- `INSUFFICIENT_BUFFER_SIZE :: 52`
- `RETURN_MAX               :: 0x100`
- `CODE_SELECTOR            :: 0x0008`
- `DATA_SELECTOR            :: 0x0010`
- `KERN_SUCCESS                  :: SUCCESS`
- `KERN_INVALID_ADDRESS          :: INVALID_ADDRESS`
- `KERN_PROTECTION_FAILURE       :: PROTECTION_FAILURE`
- `KERN_NO_SPACE                 :: NO_SPACE`
- `KERN_INVALID_ARGUMENT         :: INVALID_ARGUMENT`
- `KERN_FAILURE                  :: FAILURE`
- `KERN_RESOURCE_SHORTAGE        :: RESOURCE_SHORTAGE`
- `KERN_NOT_RECEIVER             :: NOT_RECEIVER`
- `KERN_NO_ACCESS                :: NO_ACCESS`
- `KERN_MEMORY_FAILURE           :: MEMORY_FAILURE`
- `KERN_MEMORY_ERROR             :: MEMORY_ERROR`
- `KERN_ALREADY_IN_SET           :: ALREADY_IN_SET`
- `KERN_NOT_IN_SET               :: NOT_IN_SET`
- `KERN_NAME_EXISTS              :: NAME_EXISTS`
- `KERN_ABORTED                  :: ABORTED`
- `KERN_INVALID_NAME             :: INVALID_NAME`
- `KERN_INVALID_TASK             :: INVALID_TASK`
- `KERN_INVALID_RIGHT            :: INVALID_RIGHT`
- `KERN_INVALID_VALUE            :: INVALID_VALUE`
- `KERN_UREFS_OVERFLOW           :: UREFS_OVERFLOW`
- `KERN_INVALID_CAPABILITY       :: INVALID_CAPABILITY`
- `KERN_RIGHT_EXISTS             :: RIGHT_EXISTS`
- `KERN_INVALID_HOST             :: INVALID_HOST`
- `KERN_MEMORY_PRESENT           :: MEMORY_PRESENT`
- `KERN_MEMORY_DATA_MOVED        :: MEMORY_DATA_MOVED`
- `KERN_MEMORY_RESTART_COPY      :: MEMORY_RESTART_COPY`
- `KERN_INVALID_PROCESSOR_SET    :: INVALID_PROCESSOR_SET`
- `KERN_POLICY_LIMIT             :: POLICY_LIMIT`
- `KERN_INVALID_POLICY           :: INVALID_POLICY`
- `KERN_INVALID_OBJECT           :: INVALID_OBJECT`
- `KERN_ALREADY_WAITING          :: ALREADY_WAITING`
- `KERN_DEFAULT_SET              :: DEFAULT_SET`
- `KERN_EXCEPTION_PROTECTED      :: EXCEPTION_PROTECTED`
- `KERN_INVALID_LEDGER           :: INVALID_LEDGER`
- `KERN_INVALID_MEMORY_CONTROL   :: INVALID_MEMORY_CONTROL`
- `KERN_INVALID_SECURITY         :: INVALID_SECURITY`
- `KERN_NOT_DEPRESSED            :: NOT_DEPRESSED`
- `KERN_TERMINATED               :: TERMINATED`
- `KERN_LOCK_SET_DESTROYED       :: LOCK_SET_DESTROYED`
- `KERN_LOCK_UNSTABLE            :: LOCK_UNSTABLE`
- `KERN_LOCK_OWNED               :: LOCK_OWNED`
- `KERN_LOCK_OWNED_SELF          :: LOCK_OWNED_SELF`
- `KERN_SEMAPHORE_DESTROYED      :: SEMAPHORE_DESTROYED`
- `KERN_RPC_SERVER_TERMINATED    :: RPC_SERVER_TERMINATED`
- `KERN_RPC_TERMINATE_ORPHAN     :: RPC_TERMINATE_ORPHAN`
- `KERN_RPC_CONTINUE_ORPHAN      :: RPC_CONTINUE_ORPHAN`
- `KERN_NOT_SUPPORTED            :: NOT_SUPPORTED`
- `KERN_NODE_DOWN                :: NODE_DOWN`
- `KERN_NOT_WAITING              :: NOT_WAITING`
- `KERN_OPERATION_TIMED_OUT      :: OPERATION_TIMED_OUT`
- `KERN_CODESIGN_ERROR           :: CODESIGN_ERROR`
- `KERN_POLICY_STATIC            :: POLICY_STATIC`
- `KERN_INSUFFICIENT_BUFFER_SIZE :: INSUFFICIENT_BUFFER_SIZE`
- `KERN_RETURN_MAX               :: RETURN_MAX`
- `KERN_CODE_SELECTOR            :: CODE_SELECTOR`
- `KERN_DATA_SELECTOR            :: DATA_SELECTOR`

#### `NDR_record_t` (struct)

- `mig_vers:     u8`
- `if_vers:      u8`
- `reserved1:    u8`
- `mig_encoding: u8`
- `int_rep:      u8`
- `char_rep:     u8`
- `float_rep:    u8`
- `reserved2:    u8`

#### `mach_port_status` (struct)

- `mps_pset:      mach_port_rights_t`
- `mps_seqno:     mach_port_seqno_t`
- `mps_mscount:   mach_port_mscount_t`
- `mps_qlimit:    mach_port_msgcount_t`
- `mps_msgcount:  mach_port_msgcount_t`
- `mps_sorights:  mach_port_rights_t`
- `mps_srights:   u32`
- `mps_pdrequest: u32`
- `mps_nsrequest: u32`
- `mps_flags:     u32`

#### `mach_port_limits` (struct)

- `mpl_qlimit: mach_port_msgcount_t`

#### `mach_port_info_ext` (struct)

- `mpie_status:    mach_port_status_t`
- `mpie_boost_cnt: mach_port_msgcount_t`
- `reserved:       [6] u32`

#### `mach_port_qos` (struct)

- `__bitfield: u32`
- `prealloc: u32`
- `pad1: u32; `
- `len:        u32`

#### `mach_port_options` (struct)

- `flags:    u32`
- `mpl:      mach_port_limits_t`
- `reserved: [2] u64`

#### `mach_port_guard_exception_codes` (enum_flags)

- `DESTROY         :: 0x1`
- `MOD_REFS        :: 0x2`
- `SET_CONTEXT     :: 0x4`
- `UNGUARDED       :: 0x8`
- `INCORRECT_GUARD :: 0x10`
- `kGUARD_EXC_DESTROY         :: DESTROY`
- `kGUARD_EXC_MOD_REFS        :: MOD_REFS`
- `kGUARD_EXC_SET_CONTEXT     :: SET_CONTEXT`
- `kGUARD_EXC_UNGUARDED       :: UNGUARDED`
- `kGUARD_EXC_INCORRECT_GUARD :: INCORRECT_GUARD`

#### `mach_msg_type_descriptor_t` (struct)

- `pad1:       u32`
- `pad2:       mach_msg_size_t`
- `__bitfield: u32`

#### `mach_msg_port_descriptor_t` (struct)

- `name:       mach_port_t`
- `pad1:       mach_msg_size_t`
- `__bitfield: u32`

#### `mach_msg_ool_descriptor32_t` (struct)

- `address:    u32`
- `size:       mach_msg_size_t`
- `__bitfield: u32`

#### `mach_msg_ool_descriptor64_t` (struct)

- `address:    u64 #align 4`
- `__bitfield: u32`
- `size:       mach_msg_size_t`

#### `mach_msg_ool_descriptor_t` (struct)

- `address:    *void #align 4`
- `__bitfield: u32`
- `size:       mach_msg_size_t`

#### `mach_msg_ool_ports_descriptor32_t` (struct)

- `address:    u32`
- `count:      mach_msg_size_t`
- `__bitfield: u32`

#### `mach_msg_ool_ports_descriptor64_t` (struct)

- `address:    u64 #align 4`
- `__bitfield: u32`
- `count:      mach_msg_size_t`

#### `mach_msg_ool_ports_descriptor_t` (struct)

- `address:    *void #align 4`
- `__bitfield: u32`
- `count:      mach_msg_size_t`

#### `mach_msg_descriptor_t` (struct)

- `out_of_line: mach_msg_ool_descriptor_t`
- `#overlay (out_of_line) port: mach_msg_port_descriptor_t`
- `#overlay (out_of_line) ool_ports: mach_msg_ool_ports_descriptor_t`
- `#overlay (out_of_line) type: mach_msg_type_descriptor_t`

#### `mach_msg_body_t` (struct)

- `msgh_descriptor_count: mach_msg_size_t`

#### `mach_msg_header_t` (struct)

- `msgh_bits:         mach_msg_bits_t`
- `msgh_size:         mach_msg_size_t`
- `msgh_remote_port:  mach_port_t`
- `msgh_local_port:   mach_port_t`
- `msgh_voucher_port: mach_port_name_t`
- `msgh_id:           mach_msg_id_t`

#### `mach_msg_base_t` (struct)

- `header: mach_msg_header_t`
- `body:   mach_msg_body_t`

#### `mach_msg_trailer_t` (struct)

- `msgh_trailer_type: mach_msg_trailer_type_t`
- `msgh_trailer_size: mach_msg_trailer_size_t`

#### `mach_msg_seqno_trailer_t` (struct)

- `msgh_trailer_type: mach_msg_trailer_type_t`
- `msgh_trailer_size: mach_msg_trailer_size_t`
- `msgh_seqno:        mach_port_seqno_t`

#### `security_token_t` (struct)

- `val: [2] u32`

#### `mach_msg_security_trailer_t` (struct)

- `msgh_trailer_type: mach_msg_trailer_type_t`
- `msgh_trailer_size: mach_msg_trailer_size_t`
- `msgh_seqno:        mach_port_seqno_t`
- `msgh_sender:       security_token_t`

#### `audit_token_t` (struct)

- `val: [8] u32`

#### `mach_msg_audit_trailer_t` (struct)

- `msgh_trailer_type: mach_msg_trailer_type_t`
- `msgh_trailer_size: mach_msg_trailer_size_t`
- `msgh_seqno:        mach_port_seqno_t`
- `msgh_sender:       security_token_t`
- `msgh_audit:        audit_token_t`

#### `mach_msg_context_trailer_t` (struct)

- `msgh_trailer_type: mach_msg_trailer_type_t`
- `msgh_trailer_size: mach_msg_trailer_size_t`
- `msgh_seqno:        mach_port_seqno_t`
- `msgh_sender:       security_token_t`
- `msgh_audit:        audit_token_t`
- `msgh_context:      mach_port_context_t #align 4`

#### `msg_labels_t` (struct)

- `sender: mach_port_name_t`

#### `mach_msg_mac_trailer_t` (struct)

- `msgh_trailer_type: mach_msg_trailer_type_t`
- `msgh_trailer_size: mach_msg_trailer_size_t`
- `msgh_seqno:        mach_port_seqno_t`
- `msgh_sender:       security_token_t`
- `msgh_audit:        audit_token_t`
- `msgh_context:      mach_port_context_t #align 4`
- `msgh_ad:           s32`
- `msgh_labels:       msg_labels_t`

#### `mach_msg_empty_send_t` (struct)

- `header: mach_msg_header_t`

#### `mach_msg_empty_rcv_t` (struct)

- `header:  mach_msg_header_t`
- `trailer: mach_msg_trailer_t`

#### `mach_msg_empty_t` (struct)

- `rcv: mach_msg_empty_rcv_t`
- `#overlay (rcv) send: mach_msg_empty_send_t`

#### `vm_statistics` (struct)

- `free_count:        u32`
- `active_count:      u32`
- `inactive_count:    u32`
- `wire_count:        u32`
- `zero_fill_count:   u32`
- `reactivations:     u32`
- `pageins:           u32`
- `pageouts:          u32`
- `faults:            u32`
- `cow_faults:        u32`
- `lookups:           u32`
- `hits:              u32`
- `purgeable_count:   u32`
- `purges:            u32`
- `speculative_count: u32`

#### `vm_statistics64` (struct)

- `free_count:                             u32`
- `active_count:                           u32`
- `inactive_count:                         u32`
- `wire_count:                             u32`
- `zero_fill_count:                        u64`
- `reactivations:                          u64`
- `pageins:                                u64`
- `pageouts:                               u64`
- `faults:                                 u64`
- `cow_faults:                             u64`
- `lookups:                                u64`
- `hits:                                   u64`
- `purges:                                 u64`
- `purgeable_count:                        u32`
- `speculative_count:                      u32`
- `decompressions:                         u64`
- `compressions:                           u64`
- `swapins:                                u64`
- `swapouts:                               u64`
- `compressor_page_count:                  u32`
- `throttled_count:                        u32`
- `external_page_count:                    u32`
- `internal_page_count:                    u32`
- `total_uncompressed_pages_in_compressor: u64`

#### `vm_extmod_statistics` (struct)

- `task_for_pid_count:            s64`
- `task_for_pid_caller_count:     s64`
- `thread_creation_count:         s64`
- `thread_creation_caller_count:  s64`
- `thread_set_state_count:        s64`
- `thread_set_state_caller_count: s64`

#### `vm_purgeable_stat` (struct)

- `count: u64`
- `size:  u64`

#### `vm_purgeable_info` (struct)

- `fifo_data:     [8] vm_purgeable_stat_t`
- `obsolete_data: vm_purgeable_stat_t`
- `lifo_data:     [8] vm_purgeable_stat_t`

#### `time_value` (struct)

- `seconds:      s32`
- `microseconds: s32`

#### `mach_voucher_attr_recipe_data` (struct)

- `key:              mach_voucher_attr_key_t #align 1`
- `command:          mach_voucher_attr_recipe_command_t #align 1`
- `previous_voucher: mach_voucher_name_t #align 1`
- `content_size:     mach_voucher_attr_content_size_t #align 1`

#### `policy_timeshare_base` (struct)

- `base_priority: s32`

#### `policy_timeshare_limit` (struct)

- `max_priority: s32`

#### `policy_timeshare_info` (struct)

- `max_priority:     s32`
- `base_priority:    s32`
- `cur_priority:     s32`
- `depressed:        u32`
- `depress_priority: s32`

#### `policy_rr_base` (struct)

- `base_priority: s32`
- `quantum:       s32`

#### `policy_rr_limit` (struct)

- `max_priority: s32`

#### `policy_rr_info` (struct)

- `max_priority:     s32`
- `base_priority:    s32`
- `quantum:          s32`
- `depressed:        u32`
- `depress_priority: s32`

#### `policy_fifo_base` (struct)

- `base_priority: s32`

#### `policy_fifo_limit` (struct)

- `max_priority: s32`

#### `policy_fifo_info` (struct)

- `max_priority:     s32`
- `base_priority:    s32`
- `depressed:        u32`
- `depress_priority: s32`

#### `policy_bases` (struct)

- `ts:   policy_timeshare_base_data_t`
- `rr:   policy_rr_base_data_t`
- `fifo: policy_fifo_base_data_t`

#### `policy_limits` (struct)

- `ts:   policy_timeshare_limit_data_t`
- `rr:   policy_rr_limit_data_t`
- `fifo: policy_fifo_limit_data_t`

#### `policy_infos` (struct)

- `ts:   policy_timeshare_info_data_t`
- `rr:   policy_rr_info_data_t`
- `fifo: policy_fifo_info_data_t`

#### `task_basic_info_32` (struct)

- `suspend_count: s32`
- `virtual_size:  u32`
- `resident_size: u32`
- `user_time:     time_value_t`
- `system_time:   time_value_t`
- `policy:        policy_t`

#### `task_basic_info_64` (struct)

- `suspend_count: s32`
- `virtual_size:  mach_vm_size_t #align 4`
- `resident_size: mach_vm_size_t #align 4`
- `user_time:     time_value_t`
- `system_time:   time_value_t`
- `policy:        policy_t`

#### `task_basic_info` (struct)

- `suspend_count: s32`
- `virtual_size:  vm_size_t #align 4`
- `resident_size: vm_size_t #align 4`
- `user_time:     time_value_t`
- `system_time:   time_value_t`
- `policy:        policy_t`

#### `task_events_info` (struct)

- `faults:            s32`
- `pageins:           s32`
- `cow_faults:        s32`
- `messages_sent:     s32`
- `messages_received: s32`
- `syscalls_mach:     s32`
- `syscalls_unix:     s32`
- `csw:               s32`

#### `task_thread_times_info` (struct)

- `user_time:   time_value_t`
- `system_time: time_value_t`

#### `task_absolutetime_info` (struct)

- `total_user:     u64 #align 4`
- `total_system:   u64 #align 4`
- `threads_user:   u64 #align 4`
- `threads_system: u64 #align 4`

#### `task_kernelmemory_info` (struct)

- `total_palloc: u64 #align 4`
- `total_pfree:  u64 #align 4`
- `total_salloc: u64 #align 4`
- `total_sfree:  u64 #align 4`

#### `task_affinity_tag_info` (struct)

- `set_count:  s32`
- `min:        s32`
- `max:        s32`
- `task_count: s32`

#### `task_dyld_info` (struct)

- `all_image_info_addr:   mach_vm_address_t #align 4`
- `all_image_info_size:   mach_vm_size_t #align 4`
- `all_image_info_format: s32`

#### `task_extmod_info` (struct)

- `task_uuid:         [16] u8`
- `extmod_statistics: vm_extmod_statistics_data_t #align 4`

#### `mach_task_basic_info` (struct)

- `virtual_size:      mach_vm_size_t #align 4`
- `resident_size:     mach_vm_size_t #align 4`
- `resident_size_max: mach_vm_size_t #align 4`
- `user_time:         time_value_t`
- `system_time:       time_value_t`
- `policy:            policy_t`
- `suspend_count:     s32`

#### `task_power_info` (struct)

- `total_user:                 u64 #align 4`
- `total_system:               u64 #align 4`
- `task_interrupt_wakeups:     u64 #align 4`
- `task_platform_idle_wakeups: u64 #align 4`
- `task_timer_wakeups_bin_1:   u64 #align 4`
- `task_timer_wakeups_bin_2:   u64 #align 4`

#### `task_vm_info` (struct)

- `virtual_size:                mach_vm_size_t #align 4`
- `region_count:                s32`
- `page_size:                   s32`
- `resident_size:               mach_vm_size_t #align 4`
- `resident_size_peak:          mach_vm_size_t #align 4`
- `device:                      mach_vm_size_t #align 4`
- `device_peak:                 mach_vm_size_t #align 4`
- `internal:                    mach_vm_size_t #align 4`
- `internal_peak:               mach_vm_size_t #align 4`
- `external:                    mach_vm_size_t #align 4`
- `external_peak:               mach_vm_size_t #align 4`
- `reusable:                    mach_vm_size_t #align 4`
- `reusable_peak:               mach_vm_size_t #align 4`
- `purgeable_volatile_pmap:     mach_vm_size_t #align 4`
- `purgeable_volatile_resident: mach_vm_size_t #align 4`
- `purgeable_volatile_virtual:  mach_vm_size_t #align 4`
- `compressed:                  mach_vm_size_t #align 4`
- `compressed_peak:             mach_vm_size_t #align 4`
- `compressed_lifetime:         mach_vm_size_t #align 4`
- `phys_footprint:              mach_vm_size_t #align 4`
- `min_address:                 mach_vm_address_t #align 4`
- `max_address:                 mach_vm_address_t #align 4`

#### `task_trace_memory_info` (struct)

- `user_memory_address: u64 #align 4`
- `buffer_size:         u64 #align 4`
- `mailbox_array_size:  u64 #align 4`

#### `task_wait_state_info` (struct)

- `total_wait_state_time:     u64 #align 4`
- `total_wait_sfi_state_time: u64 #align 4`
- `_reserved:                 [4] u32`

#### `gpu_energy_data` (struct)

- `task_gpu_utilisation:    u64 #align 4`
- `task_gpu_stat_reserved0: u64 #align 4`
- `task_gpu_stat_reserved1: u64 #align 4`
- `task_gpu_stat_reserved2: u64 #align 4`

#### `task_power_info_v2` (struct)

- `cpu_energy:         task_power_info_data_t`
- `gpu_energy:         gpu_energy_data`
- `task_ptime:         u64 #align 4`
- `task_pset_switches: u64 #align 4`

#### `task_flags_info` (struct)

- `flags: u32`

#### `task_inspect_flavor` (enum)

- `TASK_INSPECT_BASIC_COUNTS :: 1`

#### `task_inspect_basic_counts` (struct)

- `instructions: u64`
- `cycles:       u64`

#### `task_role` (enum)

- `RENICED                :: -1`
- `UNSPECIFIED            :: 0`
- `FOREGROUND_APPLICATION :: 1`
- `BACKGROUND_APPLICATION :: 2`
- `CONTROL_APPLICATION    :: 3`
- `GRAPHICS_SERVER        :: 4`
- `THROTTLE_APPLICATION   :: 5`
- `NONUI_APPLICATION      :: 6`
- `DEFAULT_APPLICATION    :: 7`
- `TASK_RENICED                :: RENICED`
- `TASK_UNSPECIFIED            :: UNSPECIFIED`
- `TASK_FOREGROUND_APPLICATION :: FOREGROUND_APPLICATION`
- `TASK_BACKGROUND_APPLICATION :: BACKGROUND_APPLICATION`
- `TASK_CONTROL_APPLICATION    :: CONTROL_APPLICATION`
- `TASK_GRAPHICS_SERVER        :: GRAPHICS_SERVER`
- `TASK_THROTTLE_APPLICATION   :: THROTTLE_APPLICATION`
- `TASK_NONUI_APPLICATION      :: NONUI_APPLICATION`
- `TASK_DEFAULT_APPLICATION    :: DEFAULT_APPLICATION`

#### `task_category_policy` (struct)

- `role: task_role_t`

#### `task_latency_qos` (enum)

- `UNSPECIFIED :: 0`
- `_0          :: 16711681`
- `_1          :: 16711682`
- `_2          :: 16711683`
- `_3          :: 16711684`
- `_4          :: 16711685`
- `_5          :: 16711686`
- `LATENCY_QOS_TIER_UNSPECIFIED :: UNSPECIFIED`
- `LATENCY_QOS_TIER_0           :: _0`
- `LATENCY_QOS_TIER_1           :: _1`
- `LATENCY_QOS_TIER_2           :: _2`
- `LATENCY_QOS_TIER_3           :: _3`
- `LATENCY_QOS_TIER_4           :: _4`
- `LATENCY_QOS_TIER_5           :: _5`

#### `task_throughput_qos` (enum)

- `UNSPECIFIED :: 0`
- `_0          :: 16646145`
- `_1          :: 16646146`
- `_2          :: 16646147`
- `_3          :: 16646148`
- `_4          :: 16646149`
- `_5          :: 16646150`
- `THROUGHPUT_QOS_TIER_UNSPECIFIED :: UNSPECIFIED`
- `THROUGHPUT_QOS_TIER_0           :: _0`
- `THROUGHPUT_QOS_TIER_1           :: _1`
- `THROUGHPUT_QOS_TIER_2           :: _2`
- `THROUGHPUT_QOS_TIER_3           :: _3`
- `THROUGHPUT_QOS_TIER_4           :: _4`
- `THROUGHPUT_QOS_TIER_5           :: _5`

#### `task_qos_policy` (struct)

- `task_latency_qos_tier:    task_latency_qos_t`
- `task_throughput_qos_tier: task_throughput_qos_t`

#### `thread_standard_policy` (struct)

- `no_data: u32`

#### `thread_extended_policy` (struct)

- `timeshare: u32`

#### `thread_time_constraint_policy` (struct)

- `period:      u32`
- `computation: u32`
- `constraint:  u32`
- `preemptible: u32`

#### `thread_precedence_policy` (struct)

- `importance: s32`

#### `thread_affinity_policy` (struct)

- `affinity_tag: s32`

#### `thread_background_policy` (struct)

- `priority: s32`

#### `thread_latency_qos_policy` (struct)

- `thread_latency_qos_tier: thread_latency_qos_t`

#### `thread_throughput_qos_policy` (struct)

- `thread_throughput_qos_tier: thread_throughput_qos_t`

#### `mach_timespec` (struct)

- `tv_sec:  u32`
- `tv_nsec: clock_res_t`

#### `fsid` (struct)

- `val: [2] s32`

#### `fsobj_id` (struct)

- `fid_objno:      u32`
- `fid_generation: u32`

#### `dyld_kernel_image_info` (struct)

- `uuid:      uuid_t`
- `fsobjid:   fsobj_id_t`
- `fsid:      fsid_t`
- `load_addr: u64`

#### `dyld_kernel_process_info` (struct)

- `cache_image_info:  dyld_kernel_image_info`
- `timestamp:         u64`
- `imageCount:        u32`
- `initialImageCount: u32`
- `dyldState:         u8`
- `no_cache:          u32`
- `private_cache:     u32`

#### `ipc_info_space` (struct)

- `iis_genno_mask: u32`
- `iis_table_size: u32`
- `iis_table_next: u32`
- `iis_tree_size:  u32`
- `iis_tree_small: u32`
- `iis_tree_hash:  u32`

#### `ipc_info_space_basic` (struct)

- `iisb_genno_mask:  u32`
- `iisb_table_size:  u32`
- `iisb_table_next:  u32`
- `iisb_table_inuse: u32`
- `iisb_reserved:    [2] u32`

#### `ipc_info_name` (struct)

- `iin_name:      mach_port_name_t`
- `iin_collision: s32`
- `iin_type:      mach_port_type_t`
- `iin_urefs:     mach_port_urefs_t`
- `iin_object:    u32`
- `iin_next:      u32`
- `iin_hash:      u32`

#### `ipc_info_tree_name` (struct)

- `iitn_name:   ipc_info_name_t`
- `iitn_lchild: mach_port_name_t`
- `iitn_rchild: mach_port_name_t`

#### `zone_name` (struct)

- `zn_name: [80] u8`

#### `zone_info` (struct)

- `zi_count:       s32`
- `zi_cur_size:    vm_size_t`
- `zi_max_size:    vm_size_t`
- `zi_elem_size:   vm_size_t`
- `zi_alloc_size:  vm_size_t`
- `zi_pageable:    s32`
- `zi_sleepable:   s32`
- `zi_exhaustible: s32`
- `zi_collectable: s32`

#### `mach_zone_name` (struct)

- `mzn_name: [80] u8`

#### `mach_zone_info_data` (struct)

- `mzi_count:       u64`
- `mzi_cur_size:    u64`
- `mzi_max_size:    u64`
- `mzi_elem_size:   u64`
- `mzi_alloc_size:  u64`
- `mzi_sum_size:    u64`
- `mzi_exhaustible: u64`
- `mzi_collectable: u64`

#### `task_zone_info_data` (struct)

- `tzi_count:       u64`
- `tzi_cur_size:    u64`
- `tzi_max_size:    u64`
- `tzi_elem_size:   u64`
- `tzi_alloc_size:  u64`
- `tzi_sum_size:    u64`
- `tzi_exhaustible: u64`
- `tzi_collectable: u64`
- `tzi_caller_acct: u64`
- `tzi_task_alloc:  u64`
- `tzi_task_free:   u64`

#### `mach_memory_info` (struct)

- `flags:             u64`
- `site:              u64`
- `size:              u64`
- `free:              u64`
- `largest:           u64`
- `collectable_bytes: u64`
- `mapped:            u64`
- `peak:              u64`
- `tag:               u16`
- `zone:              u16`
- `_resvA:            [2] u16`
- `_resv:             [3] u64`
- `name:              [80] u8`

#### `__Request__task_create_t` (struct)

- `Head:           mach_msg_header_t`
- `msgh_body:      mach_msg_body_t`
- `ledgers:        mach_msg_ool_ports_descriptor_t`
- `NDR:            NDR_record_t`
- `ledgersCnt:     mach_msg_type_number_t`
- `inherit_memory: u32`

#### `__Request__task_terminate_t` (struct)

- `Head: mach_msg_header_t`

#### `__Request__task_threads_t` (struct)

- `Head: mach_msg_header_t`

#### `__Request__mach_ports_register_t` (struct)

- `Head:             mach_msg_header_t`
- `msgh_body:        mach_msg_body_t`
- `init_port_set:    mach_msg_ool_ports_descriptor_t`
- `NDR:              NDR_record_t`
- `init_port_setCnt: mach_msg_type_number_t`

#### `__Request__mach_ports_lookup_t` (struct)

- `Head: mach_msg_header_t`

#### `__Request__task_info_t` (struct)

- `Head:             mach_msg_header_t`
- `NDR:              NDR_record_t`
- `flavor:           task_flavor_t`
- `task_info_outCnt: mach_msg_type_number_t`

#### `__Request__task_set_info_t` (struct)

- `Head:            mach_msg_header_t`
- `NDR:             NDR_record_t`
- `flavor:          task_flavor_t`
- `task_info_inCnt: mach_msg_type_number_t`
- `task_info_in:    [52] s32`

#### `__Request__task_suspend_t` (struct)

- `Head: mach_msg_header_t`

#### `__Request__task_resume_t` (struct)

- `Head: mach_msg_header_t`

#### `__Request__task_get_special_port_t` (struct)

- `Head:       mach_msg_header_t`
- `NDR:        NDR_record_t`
- `which_port: s32`

#### `__Request__task_set_special_port_t` (struct)

- `Head:         mach_msg_header_t`
- `msgh_body:    mach_msg_body_t`
- `special_port: mach_msg_port_descriptor_t`
- `NDR:          NDR_record_t`
- `which_port:   s32`

#### `__Request__thread_create_t` (struct)

- `Head: mach_msg_header_t`

#### `__Request__thread_create_running_t` (struct)

- `Head:         mach_msg_header_t`
- `NDR:          NDR_record_t`
- `flavor:       thread_state_flavor_t`
- `new_stateCnt: mach_msg_type_number_t`
- `new_state:    [614] u32`

#### `__Request__task_set_exception_ports_t` (struct)

- `Head:           mach_msg_header_t`
- `msgh_body:      mach_msg_body_t`
- `new_port:       mach_msg_port_descriptor_t`
- `NDR:            NDR_record_t`
- `exception_mask: exception_mask_t`
- `behavior:       exception_behavior_t`
- `new_flavor:     thread_state_flavor_t`

#### `__Request__task_get_exception_ports_t` (struct)

- `Head:           mach_msg_header_t`
- `NDR:            NDR_record_t`
- `exception_mask: exception_mask_t`

#### `__Request__task_swap_exception_ports_t` (struct)

- `Head:           mach_msg_header_t`
- `msgh_body:      mach_msg_body_t`
- `new_port:       mach_msg_port_descriptor_t`
- `NDR:            NDR_record_t`
- `exception_mask: exception_mask_t`
- `behavior:       exception_behavior_t`
- `new_flavor:     thread_state_flavor_t`

#### `__Request__lock_set_create_t` (struct)

- `Head:     mach_msg_header_t`
- `NDR:      NDR_record_t`
- `n_ulocks: s32`
- `policy:   s32`

#### `__Request__lock_set_destroy_t` (struct)

- `Head:      mach_msg_header_t`
- `msgh_body: mach_msg_body_t`
- `lock_set:  mach_msg_port_descriptor_t`

#### `__Request__semaphore_create_t` (struct)

- `Head:   mach_msg_header_t`
- `NDR:    NDR_record_t`
- `policy: s32`
- `value:  s32`

#### `__Request__semaphore_destroy_t` (struct)

- `Head:      mach_msg_header_t`
- `msgh_body: mach_msg_body_t`
- `semaphore: mach_msg_port_descriptor_t`

#### `__Request__task_policy_set_t` (struct)

- `Head:           mach_msg_header_t`
- `NDR:            NDR_record_t`
- `flavor:         task_policy_flavor_t`
- `policy_infoCnt: mach_msg_type_number_t`
- `policy_info:    [16] s32`

#### `__Request__task_policy_get_t` (struct)

- `Head:           mach_msg_header_t`
- `NDR:            NDR_record_t`
- `flavor:         task_policy_flavor_t`
- `policy_infoCnt: mach_msg_type_number_t`
- `get_default:    u32`

#### `__Request__task_sample_t` (struct)

- `Head:      mach_msg_header_t`
- `msgh_body: mach_msg_body_t`
- `reply:     mach_msg_port_descriptor_t`

#### `__Request__task_policy_t` (struct)

- `Head:      mach_msg_header_t`
- `NDR:       NDR_record_t`
- `policy:    policy_t`
- `baseCnt:   mach_msg_type_number_t`
- `base:      [5] s32`
- `set_limit: u32`
- `change:    u32`

#### `__Request__task_set_emulation_t` (struct)

- `Head:             mach_msg_header_t`
- `NDR:              NDR_record_t`
- `routine_entry_pt: vm_address_t #align 4`
- `routine_number:   s32`

#### `__Request__task_get_emulation_vector_t` (struct)

- `Head: mach_msg_header_t`

#### `__Request__task_set_emulation_vector_t` (struct)

- `Head:                mach_msg_header_t`
- `msgh_body:           mach_msg_body_t`
- `emulation_vector:    mach_msg_ool_descriptor_t`
- `NDR:                 NDR_record_t`
- `vector_start:        s32`
- `emulation_vectorCnt: mach_msg_type_number_t`

#### `__Request__task_set_ras_pc_t` (struct)

- `Head:     mach_msg_header_t`
- `NDR:      NDR_record_t`
- `basepc:   vm_address_t #align 4`
- `boundspc: vm_address_t #align 4`

#### `__Request__task_zone_info_t` (struct)

- `Head: mach_msg_header_t`

#### `__Request__task_assign_t` (struct)

- `Head:           mach_msg_header_t`
- `msgh_body:      mach_msg_body_t`
- `new_set:        mach_msg_port_descriptor_t`
- `NDR:            NDR_record_t`
- `assign_threads: u32`

#### `__Request__task_assign_default_t` (struct)

- `Head:           mach_msg_header_t`
- `NDR:            NDR_record_t`
- `assign_threads: u32`

#### `__Request__task_get_assignment_t` (struct)

- `Head: mach_msg_header_t`

#### `__Request__task_set_policy_t` (struct)

- `Head:      mach_msg_header_t`
- `msgh_body: mach_msg_body_t`
- `pset:      mach_msg_port_descriptor_t`
- `NDR:       NDR_record_t`
- `policy:    policy_t`
- `baseCnt:   mach_msg_type_number_t`
- `base:      [5] s32`
- `limitCnt:  mach_msg_type_number_t`
- `limit:     [1] s32`
- `change:    u32`

#### `__Request__task_get_state_t` (struct)

- `Head:         mach_msg_header_t`
- `NDR:          NDR_record_t`
- `flavor:       thread_state_flavor_t`
- `old_stateCnt: mach_msg_type_number_t`

#### `__Request__task_set_state_t` (struct)

- `Head:         mach_msg_header_t`
- `NDR:          NDR_record_t`
- `flavor:       thread_state_flavor_t`
- `new_stateCnt: mach_msg_type_number_t`
- `new_state:    [614] u32`

#### `__Request__task_set_phys_footprint_limit_t` (struct)

- `Head:      mach_msg_header_t`
- `NDR:       NDR_record_t`
- `new_limit: s32`

#### `__Request__task_suspend2_t` (struct)

- `Head: mach_msg_header_t`

#### `__Request__task_resume2_t` (struct)

- `Head: mach_msg_header_t`

#### `__Request__task_purgable_info_t` (struct)

- `Head: mach_msg_header_t`

#### `__Request__task_get_mach_voucher_t` (struct)

- `Head:  mach_msg_header_t`
- `NDR:   NDR_record_t`
- `which: mach_voucher_selector_t`

#### `__Request__task_set_mach_voucher_t` (struct)

- `Head:      mach_msg_header_t`
- `msgh_body: mach_msg_body_t`
- `voucher:   mach_msg_port_descriptor_t`

#### `__Request__task_swap_mach_voucher_t` (struct)

- `Head:        mach_msg_header_t`
- `msgh_body:   mach_msg_body_t`
- `new_voucher: mach_msg_port_descriptor_t`
- `old_voucher: mach_msg_port_descriptor_t`

#### `__Request__task_generate_corpse_t` (struct)

- `Head: mach_msg_header_t`

#### `__Request__task_map_corpse_info_t` (struct)

- `Head:        mach_msg_header_t`
- `msgh_body:   mach_msg_body_t`
- `corspe_task: mach_msg_port_descriptor_t`

#### `__Request__task_register_dyld_image_infos_t` (struct)

- `Head:           mach_msg_header_t`
- `msgh_body:      mach_msg_body_t`
- `dyld_images:    mach_msg_ool_descriptor_t`
- `NDR:            NDR_record_t`
- `dyld_imagesCnt: mach_msg_type_number_t`

#### `__Request__task_unregister_dyld_image_infos_t` (struct)

- `Head:           mach_msg_header_t`
- `msgh_body:      mach_msg_body_t`
- `dyld_images:    mach_msg_ool_descriptor_t`
- `NDR:            NDR_record_t`
- `dyld_imagesCnt: mach_msg_type_number_t`

#### `__Request__task_get_dyld_image_infos_t` (struct)

- `Head: mach_msg_header_t`

#### `__Request__task_register_dyld_shared_cache_image_info_t` (struct)

- `Head:             mach_msg_header_t`
- `NDR:              NDR_record_t`
- `dyld_cache_image: dyld_kernel_image_info_t #align 4`
- `no_cache:         u32`
- `private_cache:    u32`

#### `__Request__task_register_dyld_set_dyld_state_t` (struct)

- `Head:          mach_msg_header_t`
- `NDR:           NDR_record_t`
- `dyld_state:    u8`
- `dyld_statePad: [3] u8`

#### `__Request__task_register_dyld_get_process_state_t` (struct)

- `Head: mach_msg_header_t`

#### `__Request__task_map_corpse_info_64_t` (struct)

- `Head:        mach_msg_header_t`
- `msgh_body:   mach_msg_body_t`
- `corspe_task: mach_msg_port_descriptor_t`

#### `__Request__task_inspect_t` (struct)

- `Head:        mach_msg_header_t`
- `NDR:         NDR_record_t`
- `flavor:      task_inspect_flavor_t`
- `info_outCnt: mach_msg_type_number_t`

#### `__RequestUnion__task_subsystem` (struct)
- `Request_thread_create_running: __Request__thread_create_running_t; // largest member, moved to the start of the struct/union to support #overlay.`
- `#overlay (Request_thread_create_running) Request_task_create:                                __Request__task_create_t`
- `#overlay (Request_thread_create_running) Request_task_terminate:                             __Request__task_terminate_t`
- `#overlay (Request_thread_create_running) Request_task_threads:                               __Request__task_threads_t`
- `#overlay (Request_thread_create_running) Request_mach_ports_register:                        __Request__mach_ports_register_t`
- `#overlay (Request_thread_create_running) Request_mach_ports_lookup:                          __Request__mach_ports_lookup_t`
- `#overlay (Request_thread_create_running) Request_task_info:                                  __Request__task_info_t`
- `#overlay (Request_thread_create_running) Request_task_set_info:                              __Request__task_set_info_t`
- `#overlay (Request_thread_create_running) Request_task_suspend:                               __Request__task_suspend_t`
- `#overlay (Request_thread_create_running) Request_task_resume:                                __Request__task_resume_t`
- `#overlay (Request_thread_create_running) Request_task_get_special_port:                      __Request__task_get_special_port_t`
- `#overlay (Request_thread_create_running) Request_task_set_special_port:                      __Request__task_set_special_port_t`
- `#overlay (Request_thread_create_running) Request_thread_create:                              __Request__thread_create_t`
- `#overlay (Request_thread_create_running) Request_task_set_exception_ports:                   __Request__task_set_exception_ports_t`
- `#overlay (Request_thread_create_running) Request_task_get_exception_ports:                   __Request__task_get_exception_ports_t`
- `#overlay (Request_thread_create_running) Request_task_swap_exception_ports:                  __Request__task_swap_exception_ports_t`
- `#overlay (Request_thread_create_running) Request_lock_set_create:                            __Request__lock_set_create_t`
- `#overlay (Request_thread_create_running) Request_lock_set_destroy:                           __Request__lock_set_destroy_t`
- `#overlay (Request_thread_create_running) Request_semaphore_create:                           __Request__semaphore_create_t`
- `#overlay (Request_thread_create_running) Request_semaphore_destroy:                          __Request__semaphore_destroy_t`
- `#overlay (Request_thread_create_running) Request_task_policy_set:                            __Request__task_policy_set_t`
- `#overlay (Request_thread_create_running) Request_task_policy_get:                            __Request__task_policy_get_t`
- `#overlay (Request_thread_create_running) Request_task_sample:                                __Request__task_sample_t`
- `#overlay (Request_thread_create_running) Request_task_policy:                                __Request__task_policy_t`
- `#overlay (Request_thread_create_running) Request_task_set_emulation:                         __Request__task_set_emulation_t`
- `#overlay (Request_thread_create_running) Request_task_get_emulation_vector:                  __Request__task_get_emulation_vector_t`
- `#overlay (Request_thread_create_running) Request_task_set_emulation_vector:                  __Request__task_set_emulation_vector_t`
- `#overlay (Request_thread_create_running) Request_task_set_ras_pc:                            __Request__task_set_ras_pc_t`
- `#overlay (Request_thread_create_running) Request_task_zone_info:                             __Request__task_zone_info_t`
- `#overlay (Request_thread_create_running) Request_task_assign:                                __Request__task_assign_t`
- `#overlay (Request_thread_create_running) Request_task_assign_default:                        __Request__task_assign_default_t`
- `#overlay (Request_thread_create_running) Request_task_get_assignment:                        __Request__task_get_assignment_t`
- `#overlay (Request_thread_create_running) Request_task_set_policy:                            __Request__task_set_policy_t`
- `#overlay (Request_thread_create_running) Request_task_get_state:                             __Request__task_get_state_t`
- `#overlay (Request_thread_create_running) Request_task_set_state:                             __Request__task_set_state_t`
- `#overlay (Request_thread_create_running) Request_task_set_phys_footprint_limit:              __Request__task_set_phys_footprint_limit_t`
- `#overlay (Request_thread_create_running) Request_task_suspend2:                              __Request__task_suspend2_t`
- `#overlay (Request_thread_create_running) Request_task_resume2:                               __Request__task_resume2_t`
- `#overlay (Request_thread_create_running) Request_task_purgable_info:                         __Request__task_purgable_info_t`
- `#overlay (Request_thread_create_running) Request_task_get_mach_voucher:                      __Request__task_get_mach_voucher_t`
- `#overlay (Request_thread_create_running) Request_task_set_mach_voucher:                      __Request__task_set_mach_voucher_t`
- `#overlay (Request_thread_create_running) Request_task_swap_mach_voucher:                     __Request__task_swap_mach_voucher_t`
- `#overlay (Request_thread_create_running) Request_task_generate_corpse:                       __Request__task_generate_corpse_t`
- `#overlay (Request_thread_create_running) Request_task_map_corpse_info:                       __Request__task_map_corpse_info_t`
- `#overlay (Request_thread_create_running) Request_task_register_dyld_image_infos:             __Request__task_register_dyld_image_infos_t`
- `#overlay (Request_thread_create_running) Request_task_unregister_dyld_image_infos:           __Request__task_unregister_dyld_image_infos_t`
- `#overlay (Request_thread_create_running) Request_task_get_dyld_image_infos:                  __Request__task_get_dyld_image_infos_t`
- `#overlay (Request_thread_create_running) Request_task_register_dyld_shared_cache_image_info: __Request__task_register_dyld_shared_cache_image_info_t`
- `#overlay (Request_thread_create_running) Request_task_register_dyld_set_dyld_state:          __Request__task_register_dyld_set_dyld_state_t`
- `#overlay (Request_thread_create_running) Request_task_register_dyld_get_process_state:       __Request__task_register_dyld_get_process_state_t`
- `#overlay (Request_thread_create_running) Request_task_map_corpse_info_64:                    __Request__task_map_corpse_info_64_t`
- `#overlay (Request_thread_create_running) Request_task_inspect:                               __Request__task_inspect_t`

#### `__Reply__task_create_t` (struct)

- `Head:       mach_msg_header_t`
- `msgh_body:  mach_msg_body_t`
- `child_task: mach_msg_port_descriptor_t`

#### `__Reply__task_terminate_t` (struct)

- `Head:    mach_msg_header_t`
- `NDR:     NDR_record_t`
- `RetCode: kern_return_t`

#### `__Reply__task_threads_t` (struct)

- `Head:        mach_msg_header_t`
- `msgh_body:   mach_msg_body_t`
- `act_list:    mach_msg_ool_ports_descriptor_t`
- `NDR:         NDR_record_t`
- `act_listCnt: mach_msg_type_number_t`

#### `__Reply__mach_ports_register_t` (struct)

- `Head:    mach_msg_header_t`
- `NDR:     NDR_record_t`
- `RetCode: kern_return_t`

#### `__Reply__mach_ports_lookup_t` (struct)

- `Head:             mach_msg_header_t`
- `msgh_body:        mach_msg_body_t`
- `init_port_set:    mach_msg_ool_ports_descriptor_t`
- `NDR:              NDR_record_t`
- `init_port_setCnt: mach_msg_type_number_t`

#### `__Reply__task_info_t` (struct)

- `Head:             mach_msg_header_t`
- `NDR:              NDR_record_t`
- `RetCode:          kern_return_t`
- `task_info_outCnt: mach_msg_type_number_t`
- `task_info_out:    [52] s32`

#### `__Reply__task_set_info_t` (struct)

- `Head:    mach_msg_header_t`
- `NDR:     NDR_record_t`
- `RetCode: kern_return_t`

#### `__Reply__task_suspend_t` (struct)

- `Head:    mach_msg_header_t`
- `NDR:     NDR_record_t`
- `RetCode: kern_return_t`

#### `__Reply__task_resume_t` (struct)

- `Head:    mach_msg_header_t`
- `NDR:     NDR_record_t`
- `RetCode: kern_return_t`

#### `__Reply__task_get_special_port_t` (struct)

- `Head:         mach_msg_header_t`
- `msgh_body:    mach_msg_body_t`
- `special_port: mach_msg_port_descriptor_t`

#### `__Reply__task_set_special_port_t` (struct)

- `Head:    mach_msg_header_t`
- `NDR:     NDR_record_t`
- `RetCode: kern_return_t`

#### `__Reply__thread_create_t` (struct)

- `Head:      mach_msg_header_t`
- `msgh_body: mach_msg_body_t`
- `child_act: mach_msg_port_descriptor_t`

#### `__Reply__thread_create_running_t` (struct)

- `Head:      mach_msg_header_t`
- `msgh_body: mach_msg_body_t`
- `child_act: mach_msg_port_descriptor_t`

#### `__Reply__task_set_exception_ports_t` (struct)

- `Head:    mach_msg_header_t`
- `NDR:     NDR_record_t`
- `RetCode: kern_return_t`

#### `__Reply__task_get_exception_ports_t` (struct)

- `Head:          mach_msg_header_t`
- `msgh_body:     mach_msg_body_t`
- `old_handlers:  [32] mach_msg_port_descriptor_t`
- `NDR:           NDR_record_t`
- `masksCnt:      mach_msg_type_number_t`
- `masks:         [32] exception_mask_t`
- `old_behaviors: [32] exception_behavior_t`
- `old_flavors:   [32] thread_state_flavor_t`

#### `__Reply__task_swap_exception_ports_t` (struct)

- `Head:          mach_msg_header_t`
- `msgh_body:     mach_msg_body_t`
- `old_handlerss: [32] mach_msg_port_descriptor_t`
- `NDR:           NDR_record_t`
- `masksCnt:      mach_msg_type_number_t`
- `masks:         [32] exception_mask_t`
- `old_behaviors: [32] exception_behavior_t`
- `old_flavors:   [32] thread_state_flavor_t`

#### `__Reply__lock_set_create_t` (struct)

- `Head:         mach_msg_header_t`
- `msgh_body:    mach_msg_body_t`
- `new_lock_set: mach_msg_port_descriptor_t`

#### `__Reply__lock_set_destroy_t` (struct)

- `Head:    mach_msg_header_t`
- `NDR:     NDR_record_t`
- `RetCode: kern_return_t`

#### `__Reply__semaphore_create_t` (struct)

- `Head:      mach_msg_header_t`
- `msgh_body: mach_msg_body_t`
- `semaphore: mach_msg_port_descriptor_t`

#### `__Reply__semaphore_destroy_t` (struct)

- `Head:    mach_msg_header_t`
- `NDR:     NDR_record_t`
- `RetCode: kern_return_t`

#### `__Reply__task_policy_set_t` (struct)

- `Head:    mach_msg_header_t`
- `NDR:     NDR_record_t`
- `RetCode: kern_return_t`

#### `__Reply__task_policy_get_t` (struct)

- `Head:           mach_msg_header_t`
- `NDR:            NDR_record_t`
- `RetCode:        kern_return_t`
- `policy_infoCnt: mach_msg_type_number_t`
- `policy_info:    [16] s32`
- `get_default:    u32`

#### `__Reply__task_sample_t` (struct)

- `Head:    mach_msg_header_t`
- `NDR:     NDR_record_t`
- `RetCode: kern_return_t`

#### `__Reply__task_policy_t` (struct)

- `Head:    mach_msg_header_t`
- `NDR:     NDR_record_t`
- `RetCode: kern_return_t`

#### `__Reply__task_set_emulation_t` (struct)

- `Head:    mach_msg_header_t`
- `NDR:     NDR_record_t`
- `RetCode: kern_return_t`

#### `__Reply__task_get_emulation_vector_t` (struct)

- `Head:                mach_msg_header_t`
- `msgh_body:           mach_msg_body_t`
- `emulation_vector:    mach_msg_ool_descriptor_t`
- `NDR:                 NDR_record_t`
- `vector_start:        s32`
- `emulation_vectorCnt: mach_msg_type_number_t`

#### `__Reply__task_set_emulation_vector_t` (struct)

- `Head:    mach_msg_header_t`
- `NDR:     NDR_record_t`
- `RetCode: kern_return_t`

#### `__Reply__task_set_ras_pc_t` (struct)

- `Head:    mach_msg_header_t`
- `NDR:     NDR_record_t`
- `RetCode: kern_return_t`

#### `__Reply__task_zone_info_t` (struct)

- `Head:      mach_msg_header_t`
- `msgh_body: mach_msg_body_t`
- `names:     mach_msg_ool_descriptor_t`
- `info:      mach_msg_ool_descriptor_t`
- `NDR:       NDR_record_t`
- `namesCnt:  mach_msg_type_number_t`
- `infoCnt:   mach_msg_type_number_t`

#### `__Reply__task_assign_t` (struct)

- `Head:    mach_msg_header_t`
- `NDR:     NDR_record_t`
- `RetCode: kern_return_t`

#### `__Reply__task_assign_default_t` (struct)

- `Head:    mach_msg_header_t`
- `NDR:     NDR_record_t`
- `RetCode: kern_return_t`

#### `__Reply__task_get_assignment_t` (struct)

- `Head:         mach_msg_header_t`
- `msgh_body:    mach_msg_body_t`
- `assigned_set: mach_msg_port_descriptor_t`

#### `__Reply__task_set_policy_t` (struct)

- `Head:    mach_msg_header_t`
- `NDR:     NDR_record_t`
- `RetCode: kern_return_t`

#### `__Reply__task_get_state_t` (struct)

- `Head:         mach_msg_header_t`
- `NDR:          NDR_record_t`
- `RetCode:      kern_return_t`
- `old_stateCnt: mach_msg_type_number_t`
- `old_state:    [614] u32`

#### `__Reply__task_set_state_t` (struct)

- `Head:    mach_msg_header_t`
- `NDR:     NDR_record_t`
- `RetCode: kern_return_t`

#### `__Reply__task_set_phys_footprint_limit_t` (struct)

- `Head:      mach_msg_header_t`
- `NDR:       NDR_record_t`
- `RetCode:   kern_return_t`
- `old_limit: s32`

#### `__Reply__task_suspend2_t` (struct)

- `Head:          mach_msg_header_t`
- `msgh_body:     mach_msg_body_t`
- `suspend_token: mach_msg_port_descriptor_t`

#### `__Reply__task_resume2_t` (struct)

- `Head:    mach_msg_header_t`
- `NDR:     NDR_record_t`
- `RetCode: kern_return_t`

#### `__Reply__task_purgable_info_t` (struct)

- `Head:    mach_msg_header_t`
- `NDR:     NDR_record_t`
- `RetCode: kern_return_t`
- `stats:   task_purgable_info_t #align 4`

#### `__Reply__task_get_mach_voucher_t` (struct)

- `Head:      mach_msg_header_t`
- `msgh_body: mach_msg_body_t`
- `voucher:   mach_msg_port_descriptor_t`

#### `__Reply__task_set_mach_voucher_t` (struct)

- `Head:    mach_msg_header_t`
- `NDR:     NDR_record_t`
- `RetCode: kern_return_t`

#### `__Reply__task_swap_mach_voucher_t` (struct)

- `Head:        mach_msg_header_t`
- `msgh_body:   mach_msg_body_t`
- `old_voucher: mach_msg_port_descriptor_t`

#### `__Reply__task_generate_corpse_t` (struct)

- `Head:             mach_msg_header_t`
- `msgh_body:        mach_msg_body_t`
- `corpse_task_port: mach_msg_port_descriptor_t`

#### `__Reply__task_map_corpse_info_t` (struct)

- `Head:           mach_msg_header_t`
- `NDR:            NDR_record_t`
- `RetCode:        kern_return_t`
- `kcd_addr_begin: vm_address_t #align 4`
- `kcd_size:       u32`

#### `__Reply__task_register_dyld_image_infos_t` (struct)

- `Head:    mach_msg_header_t`
- `NDR:     NDR_record_t`
- `RetCode: kern_return_t`

#### `__Reply__task_unregister_dyld_image_infos_t` (struct)

- `Head:    mach_msg_header_t`
- `NDR:     NDR_record_t`
- `RetCode: kern_return_t`

#### `__Reply__task_get_dyld_image_infos_t` (struct)

- `Head:           mach_msg_header_t`
- `msgh_body:      mach_msg_body_t`
- `dyld_images:    mach_msg_ool_descriptor_t`
- `NDR:            NDR_record_t`
- `dyld_imagesCnt: mach_msg_type_number_t`

#### `__Reply__task_register_dyld_shared_cache_image_info_t` (struct)

- `Head:    mach_msg_header_t`
- `NDR:     NDR_record_t`
- `RetCode: kern_return_t`

#### `__Reply__task_register_dyld_set_dyld_state_t` (struct)

- `Head:    mach_msg_header_t`
- `NDR:     NDR_record_t`
- `RetCode: kern_return_t`

#### `__Reply__task_register_dyld_get_process_state_t` (struct)

- `Head:               mach_msg_header_t`
- `NDR:                NDR_record_t`
- `RetCode:            kern_return_t`
- `dyld_process_state: dyld_kernel_process_info_t #align 4`

#### `__Reply__task_map_corpse_info_64_t` (struct)

- `Head:           mach_msg_header_t`
- `NDR:            NDR_record_t`
- `RetCode:        kern_return_t`
- `kcd_addr_begin: mach_vm_address_t #align 4`
- `kcd_size:       mach_vm_size_t #align 4`

#### `__Reply__task_inspect_t` (struct)

- `Head:        mach_msg_header_t`
- `NDR:         NDR_record_t`
- `RetCode:     kern_return_t`
- `info_outCnt: mach_msg_type_number_t`
- `info_out:    [4] s32`

#### `__ReplyUnion__task_subsystem` (struct)
- `Reply_task_get_state: __Reply__task_get_state_t; // largest member, moved to the start of the struct/union to support #overlay.`
- `#overlay (Reply_task_get_state) Reply_task_create:                                __Reply__task_create_t`
- `#overlay (Reply_task_get_state) Reply_task_terminate:                             __Reply__task_terminate_t`
- `#overlay (Reply_task_get_state) Reply_task_threads:                               __Reply__task_threads_t`
- `#overlay (Reply_task_get_state) Reply_mach_ports_register:                        __Reply__mach_ports_register_t`
- `#overlay (Reply_task_get_state) Reply_mach_ports_lookup:                          __Reply__mach_ports_lookup_t`
- `#overlay (Reply_task_get_state) Reply_task_info:                                  __Reply__task_info_t`
- `#overlay (Reply_task_get_state) Reply_task_set_info:                              __Reply__task_set_info_t`
- `#overlay (Reply_task_get_state) Reply_task_suspend:                               __Reply__task_suspend_t`
- `#overlay (Reply_task_get_state) Reply_task_resume:                                __Reply__task_resume_t`
- `#overlay (Reply_task_get_state) Reply_task_get_special_port:                      __Reply__task_get_special_port_t`
- `#overlay (Reply_task_get_state) Reply_task_set_special_port:                      __Reply__task_set_special_port_t`
- `#overlay (Reply_task_get_state) Reply_thread_create:                              __Reply__thread_create_t`
- `#overlay (Reply_task_get_state) Reply_thread_create_running:                      __Reply__thread_create_running_t`
- `#overlay (Reply_task_get_state) Reply_task_set_exception_ports:                   __Reply__task_set_exception_ports_t`
- `#overlay (Reply_task_get_state) Reply_task_get_exception_ports:                   __Reply__task_get_exception_ports_t`
- `#overlay (Reply_task_get_state) Reply_task_swap_exception_ports:                  __Reply__task_swap_exception_ports_t`
- `#overlay (Reply_task_get_state) Reply_lock_set_create:                            __Reply__lock_set_create_t`
- `#overlay (Reply_task_get_state) Reply_lock_set_destroy:                           __Reply__lock_set_destroy_t`
- `#overlay (Reply_task_get_state) Reply_semaphore_create:                           __Reply__semaphore_create_t`
- `#overlay (Reply_task_get_state) Reply_semaphore_destroy:                          __Reply__semaphore_destroy_t`
- `#overlay (Reply_task_get_state) Reply_task_policy_set:                            __Reply__task_policy_set_t`
- `#overlay (Reply_task_get_state) Reply_task_policy_get:                            __Reply__task_policy_get_t`
- `#overlay (Reply_task_get_state) Reply_task_sample:                                __Reply__task_sample_t`
- `#overlay (Reply_task_get_state) Reply_task_policy:                                __Reply__task_policy_t`
- `#overlay (Reply_task_get_state) Reply_task_set_emulation:                         __Reply__task_set_emulation_t`
- `#overlay (Reply_task_get_state) Reply_task_get_emulation_vector:                  __Reply__task_get_emulation_vector_t`
- `#overlay (Reply_task_get_state) Reply_task_set_emulation_vector:                  __Reply__task_set_emulation_vector_t`
- `#overlay (Reply_task_get_state) Reply_task_set_ras_pc:                            __Reply__task_set_ras_pc_t`
- `#overlay (Reply_task_get_state) Reply_task_zone_info:                             __Reply__task_zone_info_t`
- `#overlay (Reply_task_get_state) Reply_task_assign:                                __Reply__task_assign_t`
- `#overlay (Reply_task_get_state) Reply_task_assign_default:                        __Reply__task_assign_default_t`
- `#overlay (Reply_task_get_state) Reply_task_get_assignment:                        __Reply__task_get_assignment_t`
- `#overlay (Reply_task_get_state) Reply_task_set_policy:                            __Reply__task_set_policy_t`
- `#overlay (Reply_task_get_state) Reply_task_set_state:                             __Reply__task_set_state_t`
- `#overlay (Reply_task_get_state) Reply_task_set_phys_footprint_limit:              __Reply__task_set_phys_footprint_limit_t`
- `#overlay (Reply_task_get_state) Reply_task_suspend2:                              __Reply__task_suspend2_t`
- `#overlay (Reply_task_get_state) Reply_task_resume2:                               __Reply__task_resume2_t`
- `#overlay (Reply_task_get_state) Reply_task_purgable_info:                         __Reply__task_purgable_info_t`
- `#overlay (Reply_task_get_state) Reply_task_get_mach_voucher:                      __Reply__task_get_mach_voucher_t`
- `#overlay (Reply_task_get_state) Reply_task_set_mach_voucher:                      __Reply__task_set_mach_voucher_t`
- `#overlay (Reply_task_get_state) Reply_task_swap_mach_voucher:                     __Reply__task_swap_mach_voucher_t`
- `#overlay (Reply_task_get_state) Reply_task_generate_corpse:                       __Reply__task_generate_corpse_t`
- `#overlay (Reply_task_get_state) Reply_task_map_corpse_info:                       __Reply__task_map_corpse_info_t`
- `#overlay (Reply_task_get_state) Reply_task_register_dyld_image_infos:             __Reply__task_register_dyld_image_infos_t`
- `#overlay (Reply_task_get_state) Reply_task_unregister_dyld_image_infos:           __Reply__task_unregister_dyld_image_infos_t`
- `#overlay (Reply_task_get_state) Reply_task_get_dyld_image_infos:                  __Reply__task_get_dyld_image_infos_t`
- `#overlay (Reply_task_get_state) Reply_task_register_dyld_shared_cache_image_info: __Reply__task_register_dyld_shared_cache_image_info_t`
- `#overlay (Reply_task_get_state) Reply_task_register_dyld_set_dyld_state:          __Reply__task_register_dyld_set_dyld_state_t`
- `#overlay (Reply_task_get_state) Reply_task_register_dyld_get_process_state:       __Reply__task_register_dyld_get_process_state_t`
- `#overlay (Reply_task_get_state) Reply_task_map_corpse_info_64:                    __Reply__task_map_corpse_info_64_t`
- `#overlay (Reply_task_get_state) Reply_task_inspect:                               __Reply__task_inspect_t`

#### `Kevent_Call_Flags` (enum_flags)

- `KEVENT_FLAG_NONE            :: 0x000000`
- `KEVENT_FLAG_IMMEDIATE       :: 0x000001`
- `KEVENT_FLAG_ERROR_EVENTS    :: 0x000002`

#### `Kevent64` (struct)

- `ident:  u64`
- `filter: Kevent_Filter`
- `flags:  Kevent_Flags`
- `fflags: Kevent_Fflags`
- `data:   s64`
- `udata:  *void`
- `ext:    [2] u64`

#### `Kevent_Filter` (enum)

- `READ     :: -1`
- `WRITE    :: -2`
- `AIO      :: -3`
- `VNODE    :: -4`
- `PROC     :: -5`
- `SIGNAL   :: -6`
- `TIMER    :: -7`
- `MACHPORT :: -8`
- `FS       :: -9`
- `USER     :: -10`
- `VM       :: -12`
- `EXCEPT   :: -15`

#### `Kevent_Flags` (enum_flags)

- `ADD             :: 0x0001`
- `DELETE          :: 0x0002`
- `ENABLE          :: 0x0004`
- `DISABLE         :: 0x0008`
- `ONESHOT         :: 0x0010`
- `CLEAR           :: 0x0020`
- `RECEIPT         :: 0x0040`
- `DISPATCH        :: 0x0080`
- `UDATA_SPECIFIC  :: 0x0100`
- `DISPATCH2       :: (DISPATCH | UDATA_SPECIFIC)`
- `VANISHED        :: 0x0200`
- `SYSFLAGS        :: 0xF000`
- `FLAG0           :: 0x1000`
- `FLAG1           :: 0x2000`
- `ERROR           :: 0x4000`
- `EOF             :: 0x8000`
- `POLL            :: FLAG0`
- `OOBAND          :: FLAG1`

#### `Kevent_Fflags` (enum_flags)

- `LOWAT          :: 0x00000001`
- `EXIT           :: 0x80000000`
- `FORK           :: 0x40000000`
- `EXEC           :: 0x20000000`
- `SIGNAL         :: 0x08000000`
- `EXITSTATUS     :: 0x04000000`
- `EXIT_DETAIL    :: 0x02000000`
- `PDATAMASK      :: 0x000fffff`
- `PCTRLMASK      :: (~PDATAMASK)`
- `EXIT_DETAIL_MASK   :: 0x00070000`
- `EXIT_DECRYPTFAIL   :: 0x00010000`
- `EXIT_MEMORY        :: 0x00020000`
- `EXIT_CSERROR       :: 0x00040000`
- `VM_PRESSURE                     :: 0x80000000`
- `VM_PRESSURE_TERMINATE           :: 0x40000000`
- `VM_PRESSURE_SUDDEN_TERMINATE    :: 0x20000000`
- `VM_ERROR                        :: 0x10000000`
- `SECONDS        :: 0x00000001`
- `USECONDS       :: 0x00000002`
- `NSECONDS       :: 0x00000004`
- `ABSOLUTE       :: 0x00000008`
- `LEEWAY         :: 0x00000010`
- `CRITICAL       :: 0x00000020`
- `BACKGROUND     :: 0x00000040`
- `MACH_CONTINUOUS_TIME       :: 0x00000080`
- `MACHTIME       :: 0x00000100`
- `TRIGGER        :: 0x01000000`
- `FFNOP      :: 0x00000000`
- `FFAND      :: 0x40000000`
- `FFOR       :: 0x80000000`
- `FFCOPY     :: 0xc0000000`
- `FFCTRLMASK :: 0xc0000000`
- `FFLAGSMASK :: 0x00ffffff`

#### `FSEventStreamContext` (struct)

- `copyDescription: CFAllocatorCopyDescriptionCallBack`
- `info: *void`
- `retain: CFAllocatorRetainCallBack`
- `release: CFAllocatorReleaseCallBack`
- `version: CFIndex`

This is the public Jai declaration's exact field order and types; no field has an explicit default. A zero-initialized value has null callback/info pointers and version zero. The order is ABI-significant. It differs from the native macOS SDK's `FSEventStreamContext` order, which places `version` first and `copyDescription` last; this source declaration therefore does not match the native C record layout even though `FSEventStreamCreate` accepts a pointer to it. Keep the Jai shape as written and treat this as a genuine source-to-native ABI mismatch requiring correction in the binding before passing this record to the operating system.

#### `FSRef` (struct)

`FSRef :: struct {}` is an explicitly empty public aggregate.


#### `__CFAllocator` (struct)

`__CFAllocator :: struct {}` is an explicitly empty public aggregate.


#### `__CFArray` (struct)

`__CFArray :: struct {}` is an explicitly empty public aggregate.


#### `__CFAttributedString` (struct)

`__CFAttributedString :: struct {}` is an explicitly empty public aggregate.


#### `__CFBag` (struct)

`__CFBag :: struct {}` is an explicitly empty public aggregate.


#### `__CFBinaryHeap` (struct)

`__CFBinaryHeap :: struct {}` is an explicitly empty public aggregate.


#### `__CFBitVector` (struct)

`__CFBitVector :: struct {}` is an explicitly empty public aggregate.


#### `__CFBoolean` (struct)

`__CFBoolean :: struct {}` is an explicitly empty public aggregate.


#### `__CFBundle` (struct)

`__CFBundle :: struct {}` is an explicitly empty public aggregate.


#### `__CFCalendar` (struct)

`__CFCalendar :: struct {}` is an explicitly empty public aggregate.


#### `__CFCharacterSet` (struct)

`__CFCharacterSet :: struct {}` is an explicitly empty public aggregate.


#### `__CFData` (struct)

`__CFData :: struct {}` is an explicitly empty public aggregate.


#### `__CFDate` (struct)

`__CFDate :: struct {}` is an explicitly empty public aggregate.


#### `__CFDateFormatter` (struct)

`__CFDateFormatter :: struct {}` is an explicitly empty public aggregate.


#### `__CFDictionary` (struct)

`__CFDictionary :: struct {}` is an explicitly empty public aggregate.


#### `__CFError` (struct)

`__CFError :: struct {}` is an explicitly empty public aggregate.


#### `__CFFileDescriptor` (struct)

`__CFFileDescriptor :: struct {}` is an explicitly empty public aggregate.


#### `__CFLocale` (struct)

`__CFLocale :: struct {}` is an explicitly empty public aggregate.


#### `__CFMachPort` (struct)

`__CFMachPort :: struct {}` is an explicitly empty public aggregate.


#### `__CFMessagePort` (struct)

`__CFMessagePort :: struct {}` is an explicitly empty public aggregate.


#### `__CFNotificationCenter` (struct)

`__CFNotificationCenter :: struct {}` is an explicitly empty public aggregate.


#### `__CFNull` (struct)

`__CFNull :: struct {}` is an explicitly empty public aggregate.


#### `__CFNumber` (struct)

`__CFNumber :: struct {}` is an explicitly empty public aggregate.


#### `__CFNumberFormatter` (struct)

`__CFNumberFormatter :: struct {}` is an explicitly empty public aggregate.


#### `__CFPlugInInstance` (struct)

`__CFPlugInInstance :: struct {}` is an explicitly empty public aggregate.


#### `__CFReadStream` (struct)

`__CFReadStream :: struct {}` is an explicitly empty public aggregate.


#### `__CFRunLoop` (struct)

`__CFRunLoop :: struct {}` is an explicitly empty public aggregate.


#### `__CFRunLoopObserver` (struct)

`__CFRunLoopObserver :: struct {}` is an explicitly empty public aggregate.


#### `__CFRunLoopSource` (struct)

`__CFRunLoopSource :: struct {}` is an explicitly empty public aggregate.


#### `__CFRunLoopTimer` (struct)

`__CFRunLoopTimer :: struct {}` is an explicitly empty public aggregate.


#### `__CFSet` (struct)

`__CFSet :: struct {}` is an explicitly empty public aggregate.


#### `__CFSocket` (struct)

`__CFSocket :: struct {}` is an explicitly empty public aggregate.


#### `__CFString` (struct)

`__CFString :: struct {}` is an explicitly empty public aggregate.


#### `__CFStringTokenizer` (struct)

`__CFStringTokenizer :: struct {}` is an explicitly empty public aggregate.


#### `__CFTimeZone` (struct)

`__CFTimeZone :: struct {}` is an explicitly empty public aggregate.


#### `__CFTree` (struct)

`__CFTree :: struct {}` is an explicitly empty public aggregate.


#### `__CFURL` (struct)

`__CFURL :: struct {}` is an explicitly empty public aggregate.


#### `__CFURLEnumerator` (struct)

`__CFURLEnumerator :: struct {}` is an explicitly empty public aggregate.


#### `__CFUUID` (struct)

`__CFUUID :: struct {}` is an explicitly empty public aggregate.


#### `__CFUserNotification` (struct)

`__CFUserNotification :: struct {}` is an explicitly empty public aggregate.


#### `__CFWriteStream` (struct)

`__CFWriteStream :: struct {}` is an explicitly empty public aggregate.

## Behavioral contract and constraints

This module binds Darwin/macOS system APIs, including filesystem/event and framework interfaces. Native calls return OS status/errno values and retain native ownership/lifetime rules for handles and objects. The module is macOS-specific; the source includes transcribed system interfaces, so SDK/OS version compatibility is not guaranteed beyond declared signatures.

The public surface is a direct native binding: declared records, constants, and function signatures preserve the represented external ABI; this wrapper does not itself define a different ownership or error model. Native status/error values and handle lifetimes follow the external API. ABI compatibility across external-library versions is not specified unless a version note appears above.

External libraries/frameworks named in source: `CoreFoundation`, `CoreServices`, `libc`.
