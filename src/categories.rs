//! Automatic, multilingual (EN/ES/PT/DE/RU/TR) categories from project name, path and file names.
//! A keyword ending in `*` matches as a word prefix, otherwise the whole word must match.
//! A project can belong to several categories.

pub struct Cat {
    pub slug: &'static str,
    pub label: &'static str,
    pub icon: &'static str,
    pub words: &'static [&'static str],
}

pub const CATS: &[Cat] = &[
    Cat { slug: "christmas", label: "Christmas & Winter", icon: "🎄", words: &[
        "christmas*", "cristmas*", "xmas", "navid*", "natal*", "noel", "weihnacht*", "santa", "claus",
        "nativity", "nativid*", "presepio", "pesebre", "snowflake*", "snowman", "flocos", "copos",
        "reindeer", "rena", "renos", "gingerbread", "advent*", "ёлк*", "новогод*", "рождеств*", "winter", "inverno", "invierno"] },
    Cat { slug: "halloween", label: "Halloween", icon: "🎃", words: &[
        "halloween*", "bruxa*", "bruja*", "witch*", "pumpkin*", "calabaza*", "abobora*", "ghost*", "fantasma*", "skull*", "calavera*", "caveira*", "spooky", "vampir*"] },
    Cat { slug: "easter", label: "Easter & Spring", icon: "🐣", words: &[
        "easter", "pascua*", "pascoa*", "páscoa*", "ostern", "bunny", "bunnies", "coelh*", "conejo*", "spring", "primavera"] },
    Cat { slug: "love", label: "Love & Wedding", icon: "💕", words: &[
        "love", "valentin*", "namorad*", "heart*", "coraz*", "coraç*", "coracao*", "wedding*", "boda*", "casament*",
        "hochzeit*", "amor", "cupid*", "bride", "noiv*", "anniversar*", "aniversario*"] },
    Cat { slug: "party", label: "Party & Celebration", icon: "🎉", words: &[
        "birthday*", "cumplea*", "geburtstag*", "party", "fiesta*", "festa*", "celebrat*", "cake", "cupcake*", "topper*",
        "baby", "bebe", "bebé", "shower", "graduat*", "formatura*", "candy", "candybar*", "cha", "baptism", "bautizo*", "batizado*"] },
    Cat { slug: "cards", label: "Cards, Invitations & Packaging", icon: "💌", words: &[
        "card", "cards", "cartão", "cartões", "cartoes", "cartao", "tarjet*", "invitation*", "convite*", "invitaci*",
        "envelope*", "sobre*", "embalage*", "packaging", "caixinha*", "place_cards", "bookmark*", "marcador*", "tag", "tags"] },
    Cat { slug: "animals", label: "Animals", icon: "🦊", words: &[
        "animal*", "animai*", "dog", "dogs", "perro*", "cachorr*", "puppy", "cat", "cats", "gato*", "kitten", "horse*", "caball*",
        "caval*", "bird*", "pajaro*", "passaro*", "pássar*", "owl*", "buho*", "coruja*", "eagle*", "aguia*", "aguila*", "bear*",
        "oso", "osos", "urso*", "wolf", "wolves", "lobo*", "fox", "foxes", "zorro*", "raposa*", "deer", "ciervo*", "venado*",
        "elephant*", "elefant*", "lion*", "leon", "leão", "leao", "tiger*", "tigre*", "fish*", "pez", "peces", "peixe*", "shark*",
        "tubar*", "dolphin*", "delfin*", "golfinh*", "butterfl*", "borboleta*", "mariposa*", "hummingbird*", "colibri*", "beija",
        "rabbit*", "turtle*", "tortuga*", "tartaruga*", "giraffe*", "jirafa*", "girafa*", "monkey*", "mono", "macaco*",
        "gorilla*", "rhino*", "rinocer*", "cow", "vaca*", "pig", "cerdo*", "porco*", "frog*", "rana", "sapo*", "squirrel*",
        "esquilo*", "ardilla*", "panda*", "koala*", "penguin*", "pinguin*", "pingüin*", "unicorn*", "unicornio*", "bee", "bees",
        "abeja*", "abelha*", "insect*", "spider*", "dragonfly", "ladybug", "parrot*", "flamingo*", "swan*", "cisne*", "duck*", "pato*",
        "hen", "chicken*", "gallina*", "galinha*", "rooster", "gallo", "galo", "camel*", "zebra*", "kangaroo*", "octopus", "pulpo",
        "polvo", "whale*", "ballena*", "baleia*", "crab*", "seahorse*", "hirsch", "hund", "katze", "pferd", "vogel", "eule"] },
    Cat { slug: "dinosaurs", label: "Dinosaurs & Dragons", icon: "🦖", words: &[
        "dino", "dinos", "dinosau*", "dinossau*", "t-rex", "trex", "rex", "raptor*", "triceratop*", "stegosaur*", "brachiosaur*",
        "pterodact*", "pteradact*", "dragon*", "dragão", "dragões", "dragoes", "drag", "dragones", "drache*", "esqueleto*", "skeleton*"] },
    Cat { slug: "vehicles", label: "Cars & Vehicles", icon: "🚗", words: &[
        "car", "cars", "carro*", "coche*", "auto", "autos", "truck*", "camion*", "caminh*", "jeep*", "pickup*", "bus", "onibus",
        "tractor*", "trator*", "motorcycle*", "motorbike*", "moto", "motos", "harley*", "bike", "bikes", "bicycle*", "bicicleta*",
        "scania*", "mercedes*", "porsche*", "ferrari*", "volkswagen*", "beetle", "fusca*", "chevette", "chevrolet*", "ford", "bmw",
        "hummer*", "land", "cruiser*", "defender*", "willys", "rally*", "racing", "race", "formula", "tanker", "tank", "tanque*",
        "excavator*", "bulldozer*", "buldozer*", "loader", "crane*", "forklift*", "sidecar*", "carriage*", "wagon*", "carruaje*",
        "train*", "tren", "trem", "locomotiv*", "vehicle*", "vehiculo*", "veiculo*", "veículo*", "lkw", "traktor*"] },
    Cat { slug: "air-sea", label: "Planes, Ships & Space", icon: "✈️", words: &[
        "plane*", "airplane*", "aircraft", "avion*", "avião", "aviao", "helicopter*", "helicoptero*", "helicóptero*", "apache",
        "jet", "fighter", "drone*", "dron", "ship*", "boat*", "barco*", "navio*", "sailboat", "yacht", "submarine*", "pirate*",
        "pirata*", "titanic", "rocket*", "cohete*", "foguete*", "space", "spaceship", "planet*", "astronaut*", "nave*", "star_wars", "wars"] },
    Cat { slug: "buildings", label: "Buildings & Architecture", icon: "🏰", words: &[
        "house*", "casa", "casas", "casita*", "haus", "home", "castle*", "castillo*", "castelo*", "chateau*", "fort", "fortress*",
        "church*", "iglesia*", "igreja*", "cathedral*", "catedral*", "notre", "mahal", "taj", "tower*", "torre*", "eiffel",
        "lighthouse*", "farol", "faro", "villa", "villa_rev", "building*", "edificio*", "skyline*", "city", "ciudad", "cidade",
        "barn", "cabin*", "dollhouse*", "muñeca*", "mu•ecas", "bonecas", "windmill*", "molino*", "moinho*", "bridge*", "puente*",
        "temple*", "templo*", "pagoda*", "charminar", "parthenon", "colosseum", "kremlin", "tea_house*", "birdhouse*", "casinha*", "gate"] },
    Cat { slug: "furniture", label: "Furniture", icon: "🪑", words: &[
        "furniture*", "mueble*", "movel", "moveis", "móveis", "möbel", "chair*", "silla*", "cadeira*", "stool*", "banco*", "table",
        "tables", "mesa", "mesas", "mesita*", "bench*", "bed", "cama", "cabinet*", "armario*", "dresser", "sofa*", "shelf", "shelves",
        "estante*", "prateleira*", "repisa*", "bookcase*", "bookshelf*", "rack", "racks", "nightstand*", "cradle", "cuna", "berço"] },
    Cat { slug: "lighting", label: "Lamps & Lighting", icon: "💡", words: &[
        "lamp*", "lampara*", "lámpara*", "lampada*", "luminaria*", "lampe*", "light", "lights", "lighting", "led", "3dlamp*",
        "chandelier*", "lustre*", "candle*", "vela", "velas", "kerze*", "lantern*", "linterna*", "lanterna*", "laterne*",
        "nightlight*", "abajur*", "лампы", "лампа*", "светильник*", "lambalar*", "lamba*", "hanging", "pendant", "sconce*"] },
    Cat { slug: "clocks", label: "Clocks", icon: "🕰️", words: &[
        "clock*", "reloj*", "relogio*", "relógio*", "uhr", "uhren", "chasy", "часы", "watch", "pendulum"] },
    Cat { slug: "storage", label: "Boxes & Storage", icon: "📦", words: &[
        "box", "boxes", "caja*", "caixa*", "kiste*", "schachtel*", "chest*", "cofre*", "baul*", "baú", "jewelry", "jewellery",
        "joyero*", "porta-joias", "tissue", "pañuelo*", "storage", "drawer*", "cajon*", "gaveta*", "basket*", "cesta*", "canasta*",
        "crate*", "case", "estuche*", "tray*", "bandeja*", "coin", "piggy", "alcancia*", "cofrinho*", "tea", "teabox", "urn*"] },
    Cat { slug: "organizers", label: "Organizers & Holders", icon: "🗂️", words: &[
        "organizer*", "organizador*", "holder*", "porta", "portas", "soporte*", "suporte*", "halter*", "stand", "stands",
        "desk", "desktop", "pen", "pencil*", "caneta*", "lapicero*", "phone*", "celular*", "movil", "tablet*", "key", "keys",
        "llaves", "chaves", "napkin*", "servillet*", "guardanapo*", "dispenser*", "makeup", "cosmetic*", "headphone*", "remote",
        "mail", "letterbox", "business", "card_holder", "brush*", "hanger*", "hook*", "gancho*", "cabide*", "perchero*"] },
    Cat { slug: "kitchen", label: "Kitchen & Bar", icon: "🍷", words: &[
        "kitchen*", "cocina*", "cozinha*", "küche", "wine*", "vino*", "vinho*", "wein*", "beer*", "cerveza*", "cerveja*", "bier*",
        "bottle*", "botella*", "garrafa*", "glass", "glasses", "copa*", "taça*", "coaster*", "posavaso*", "portacopo*",
        "cutting", "board", "tabla*", "spice*", "especiero*", "tempero*", "bread*", "pan", "egg*", "huevo*", "ovo", "ovos",
        "cup", "cups", "mug*", "taza*", "caneca*", "coffee*", "cafe", "café", "kaffee", "tea", "té", "minibar", "bar"] },
    Cat { slug: "toys", label: "Toys & Games", icon: "🧸", words: &[
        "toy", "toys", "juguete*", "brinquedo*", "spielzeug*", "game*", "juego*", "jogo*", "chess", "ajedrez", "xadrez", "dice",
        "domino*", "kids", "kid", "niños", "nino*", "infantil*", "child*", "crian*", "doll*", "boneca*", "rocking", "balancin*",
        "marble*", "maze*", "labirinto*", "laberinto*", "carousel*", "carrossel*", "carrusel*", "yo-yo", "spinner*", "mechanical",
        "mecanico*", "mecânico*", "automata*", "gear*", "engrenage*", "engranaje*", "catapult*", "trebuchet*", "gun", "guns", "rifle*", "pistol*", "arma", "armas"] },
    Cat { slug: "puzzles", label: "3D Puzzles & Assembly Models", icon: "🧩", words: &[
        "puzzle*", "puzle*", "quebra", "quebra-cabeça*", "rompecabeza*", "assembly", "assy", "_assy", "armable*", "encastrable*",
        "encaixe*", "maqueta*", "maquete*", "kit", "sheetmetal", "metalcraftdesign", "snapshot*", "60premium*"] },
    Cat { slug: "multilayer", label: "Multilayer & Shadow Boxes", icon: "🗻", words: &[
        "multilayer*", "multi-layer", "layered", "layer", "layers", "capas", "camadas", "shadow", "shadowbox*", "paperlaser",
        "paper", "papercut*", "lightbox*", "diorama*", "3d-layered", "mountain*", "montaña*", "montanha*"] },
    Cat { slug: "mandalas", label: "Mandalas & Ornaments", icon: "🌀", words: &[
        "mandala*", "ornament*", "ornamento*", "arabesc*", "arabesque*", "celtic*", "celtico*", "céltico*", "pattern*", "patron*",
        "padr*", "geometric*", "geometri*", "geometry", "filigree", "lace", "renda*", "encaje*", "damask", "border*", "borda*",
        "borde*", "corner*", "esquina*", "cantoneira*", "trim", "trim_corner", "fretwork", "tile*", "azulejo*", "circle*", "circulo*", "rosette*"] },
    Cat { slug: "nature", label: "Flowers, Trees & Nature", icon: "🌿", words: &[
        "flower*", "flor", "flores", "floral*", "florais", "blume*", "rose", "roses", "sunflower*", "girassol*", "girasol*",
        "tulip*", "lotus", "leaf", "leaves", "hoja*", "folha*", "folhage*", "tree*", "arbol*", "árbol*", "arvore*", "árvore*",
        "baum*", "derevo*", "дерев*", "galho*", "rama*", "branch*", "plant*", "planta*", "cactus*", "succulent*", "landscape*",
        "paisaje*", "paisage*", "forest*", "bosque*", "floresta*", "sun", "sol", "moon", "luna", "lua", "clima", "cloud*", "nuvem*", "nuvens", "nube*"] },
    Cat { slug: "spiritual", label: "Religious & Spiritual", icon: "✝️", words: &[
        "relig*", "cross", "crosses", "cruz", "cruces", "kreuz*", "jesus", "jesús", "cristo*", "christ", "virgen", "virgin",
        "maria", "mary", "angel*", "anjo*", "engel*", "saint*", "santo*", "santa_*", "guadalupe", "biblia*", "bible", "prayer",
        "oracion*", "oração", "rosary", "rosario*", "padre", "nuestro", "chakra*", "spiritual*", "sacred*", "mystic*", "zodiac*",
        "zodiaco*", "horoscop*", "buddha*", "buda", "om", "hamsa", "tarot*", "dreamcatcher*", "catcher", "atrapasue*", "witchcraft", "star", "stars", "estrella*", "estrela*"] },
    Cat { slug: "letters", label: "Letters, Signs & Names", icon: "🔤", words: &[
        "letter*", "letra*", "buchstab*", "alphabet*", "alfabet*", "abecedario*", "abc", "font*", "fuente*", "fonte*", "number*",
        "numero*", "número*", "name", "names", "nombre*", "nome", "nomes", "monogram*", "sign", "signs", "cartel*", "letrero*",
        "placa*", "schild*", "quote*", "frase*", "word*", "palabra*", "palavra*", "text", "texto", "welcome", "bienvenid*", "door_hanger*", "door-knob-hanger*", "doorhanger*", "logo*", "logotipo*", "escudo*", "emblem*"] },
    Cat { slug: "wall-decor", label: "Wall Art & Decor", icon: "🖼️", words: &[
        "wall", "walls", "pared*", "parede*", "wand*", "decor*", "decora*", "decoração", "deko*", "arte", "frame*",
        "marco*", "moldura*", "rahmen*", "рамк*", "ramk*", "ramka*", "portaretrat*", "portarretrat*", "photo", "mirror*", "espejo*",
        "espelho*", "spiegel*", "panel*", "painel*", "paine*", "screen*", "biombo*", "divider*", "partition*", "silhouette*",
        "silueta*", "silhueta*", "portrait*", "retrato*", "people", "figura*", "figure*", "vinilo*", "vinil*", "sticker*", "clipart*", "cartoon*", "dibujo*", "desenho*"] },
    Cat { slug: "jewelry", label: "Jewelry & Accessories", icon: "💍", words: &[
        "earring*", "pendiente*", "brinco*", "aretes", "ohrring*", "necklace*", "collar*", "colar*", "pendant", "pingente*",
        "bracelet*", "pulsera*", "pulseira*", "keychain*", "llavero*", "chaveiro*", "keyring*", "ring", "anillo*", "anel",
        "brooch*", "broche*", "hairclip*", "comb", "combs", "peine*", "beard", "bow", "tie", "gravata*", "medal*", "medalla*"] },
    Cat { slug: "sports-music", label: "Sports, Music & Hobbies", icon: "⚽", words: &[
        "sport*", "esporte*", "deporte*", "futbol*", "fútbol*", "futebol*", "football*", "soccer", "basketball*", "baseball*",
        "tennis*", "golf*", "ski*", "surf*", "skate*", "skae", "bowling", "gym", "fitness", "yoga", "music*", "musica*", "música*",
        "guitar*", "guitarra*", "violin*", "piano*", "drum*", "nota*", "note*", "instrument*", "fishing*", "pesca*", "camping",
        "hunting", "caza", "chess", "team", "shirts", "camiseta*", "camisa*", "jersey", "trophy", "trofeo*", "troféu*"] },
    Cat { slug: "tools", label: "Workshop, Tools & Templates", icon: "🛠️", words: &[
        "tool*", "herramienta*", "ferramenta*", "werkzeug*", "jig*", "template*", "plantilla*", "gabarito*", "molde*",
        "stencil*", "estencil*", "hinge*", "bisagra*", "joint*", "box_joint", "finger", "living", "flex*", "flexible*",
        "kerf*", "bending", "superficie*", "calibration", "calibra*", "engineering", "engenharia*", "mechanism*", "bracket*",
        "calendar*", "calendario*", "ruler*", "regla*", "regua*", "régua*", "workshop", "taller", "oficina"] },
];

use std::collections::HashMap;
use std::sync::OnceLock;

/// keyword (without '*') -> (is_prefix, category index)
fn lookup() -> &'static HashMap<&'static str, Vec<(bool, usize)>> {
    static MAP: OnceLock<HashMap<&'static str, Vec<(bool, usize)>>> = OnceLock::new();
    MAP.get_or_init(|| {
        let mut m: HashMap<&'static str, Vec<(bool, usize)>> = HashMap::new();
        for (ci, c) in CATS.iter().enumerate() {
            for kw in c.words {
                let (k, prefix) = match kw.strip_suffix('*') { Some(p) => (p, true), None => (*kw, false) };
                m.entry(k).or_default().push((prefix, ci));
            }
        }
        m
    })
}

/// Returns the slugs of all categories whose keywords occur in `text` (already lowercased).
pub fn categorize(text: &str) -> Vec<&'static str> {
    let map = lookup();
    let mut hit = vec![false; CATS.len()];
    // words split on everything non-alphanumeric, plus '_'/'-' joined variants ("trim_corner", "t-rex")
    let split = text.split(|c: char| !c.is_alphanumeric());
    let joined = text.split(|c: char| !(c.is_alphanumeric() || c == '_' || c == '-'));
    for w in split.chain(joined).filter(|w| !w.is_empty()) {
        // every char-boundary prefix of the word is a candidate key
        for (end, _) in w.char_indices().skip(1).chain(std::iter::once((w.len(), ' '))) {
            if let Some(v) = map.get(&w[..end]) {
                let whole = end == w.len();
                for &(prefix, ci) in v {
                    if prefix || whole { hit[ci] = true; }
                }
            }
        }
    }
    CATS.iter().zip(hit).filter(|(_, h)| *h).map(|(c, _)| c.slug).collect()
}
