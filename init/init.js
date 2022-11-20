db.products.insertMany([
    {
        _id: "dumplings",
        default: 25,
    },
    {
        _id: "ice",
        default: 20,
    },
    {
        _id: "icecream",
        default: 1,
    },
    {
        _id: "mapo-tofu",
        default: 7,
    },
    {
        _id: "minced meat",
        default: 4,
    },
    {
        _id: "chicken",
        default: 2,
    },
])

db.freezers.createIndex({"_id": 1, "products.product_id": 1}, {unique: true})

db.freezers.insertMany([
    {
        "_id": "ATLANT М 7184-003",
        "owner": "ИП Серегин",
        "products": {
            "ice": 5,
        },
        "model": {
            "name": "Frier",
            "year": 2012
        }
    },
    {
        "_id": "Samsung SM93924H3",
        "owner": "ИП Борис",
        "products": {
            "minced meat": 220,
            "icecream": 1,
        },
        "model": {
            "name": "Monster",
            "year": 2020
        }
    },
    {
        "_id": "Horizont M2",
        "owner": "МС Спринг",
        "products": {
            "dumplings": 23
        },
        "model": {
            "name": "Lol",
            "year": 2010
        }
    },
    {
        "_id": "ATLANT М3223-R",
        "owner": "Кор",
        "products": {},
        "model": {
            "name": "Rider",
            "year": 2011
        }
    },
    {
        "_id": "Panasonic ultra cool",
        "owner": "ИП Борис",
        "products": {
            "ice": 1,
            "chicken": 2,
            "mapo-tofu": 4,
        },
        "model": {
            "name": "Cooler Master",
            "year": 2022
        }
    }
])