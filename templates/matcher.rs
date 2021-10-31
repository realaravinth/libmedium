match p.type_.as_str() {
   "IMG" => { 
        include!("./img.html");
   },
   _ => unimplemented!(),

}

//<. match p.type_ { .>
//          <. "IMG" => { .>
//            <. include!("./img.html") .>
//            <. }, .>
//          <. _ => log::error("Unable to find paragraph render class. Post ID: {}. Paragraph item {:?}", id, p),
//        <. } .>
//
