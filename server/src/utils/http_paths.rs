pub fn get_root_path(src: &[u8]) -> Option<&[u8]> {
    let src = src.strip_prefix(b"/")?;
    let root = src
        .iter()
        .position(|&u| u == b'/')
        .map_or(src, |pos| &src[..pos]);
    Some(
        root.iter()
            .position(|&u| u == b'?')
            .map_or(root, |quest_pos| /* Strip query */ &root[..quest_pos]),
    )
}
