/** @brief Adds values.
 * Continued summary.
 *
 * @param[in] x first value
 * \param y second value
 * @return their sum
 * @returns the same sum
 * \return a number
 * @note Keep this note.
 * @warning Check overflow.
 * @deprecated Use another function.
 * @see other
 * @sa related
 * @mystery retained text
 * \unknown retained too
 * @code
 * int value = 1;
 *     value += 2;
 *     int example_with_a_long_name = value + 123456789;
 * @endcode
 */
int documented(int x, int y);
/*! Block bang. */
struct Record {
    int value; ///< Stored value.
    int other; //!< Other value.
    int long_note; ///< A trailing member comment that spans several words and wraps with the declaration width.
};
/// Slash docs.
//! Bang docs.
/// A paragraph with enough words to wrap at the formatter width while keeping the text in its original order without losing any words.
struct Lines { int field; };
struct Undocumented { int field; };

// Unattached ordinary comment that clang drops.
