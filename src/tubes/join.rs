struct Join<L, R>
where
    L: Tubeable,
    R: Tubeable,
{
    left: L,
    right: R,
}
