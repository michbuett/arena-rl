Alchemy.ns('Alchemy.v');

/**
 * @class Alchemy.v.Sprite
 * @extends Alchemy.v.BasicElement
 *
 * The type representing a sprite
 */
Alchemy.v.Sprite = {

    /**
     * the image representation of the sprite
     *
     * @property image
     * @type HTMLImageElement
     */
    image: undefined,

    constructor: function (cfg) {
        _super.call(this, cfg);

        if (typeof this.image ==='string') {
            var src = this.image;
            var resource = Alchemy.resources.get(src);

            if (resource && resource.data) {
                this.setImage(resource.data);
            } else {
                Alchemy.resources.load({
                    src: src,
                    success: function (r) {
                        this.setImage(r.data);
                    },
                    scope: this
                });
            }
        }
    },

    setImage: function (img) {
        this.image = img;
        if (img && (!this.getWidth() || this.getHeight())) {
            this.setSize(img.width, img.height);
        }
        this.updateUI();
    },

    getImage: function () {
        return this.image;
    },

    render: function (ctx) {
        var img = this.getImage();
        if (img) {
            ctx.drawImage(img, this.getX(), this.getY(), this.getWidth(), this.getHeight());
        }
        return ctx;
    }
};

Alchemy.v.Sprite = Alchemy.brew({
    extend: Alchemy.v.BasicElement
}, Alchemy.v.Sprite);
