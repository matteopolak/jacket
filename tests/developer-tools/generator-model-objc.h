@protocol OriginalModelProtocol
- (int)reading;
@end
__attribute__((objc_root_class))
@interface OriginalModelRoot <OriginalModelProtocol> { @public int ignored_ivar; }
- (int)add:(int)value;
+ (int)factory;
@end
@interface OriginalModelRoot (Extras)
- (int)extra;
@end
@interface OriginalModelBox<__covariant T> : OriginalModelRoot
- (T)item;
@end
