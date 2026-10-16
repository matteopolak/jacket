#define DRIVE_CODE 7
#define FAMILY_ONE 4
#define FAMILY_TWO (FAMILY_ONE + 1)
struct Drive_Record { int first; int second; };
typedef struct Drive_Record Drive_Alias;
enum DriveStatus { DRIVE_BAD = -3, DRIVE_GOOD = 5 };
