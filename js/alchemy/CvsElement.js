Alchemy.ns('Alchemy.v');

/**
 * @class Alchemy.v.CvsElement
 * @extends Alchemy.v.BasicElement
 *
 * The type representing a canvas element
 */
Alchemy.v.CvsElement = {

    constructor: function (cfg) {
        _super.call(this, cfg);

        if (typeof this.image ==='string') {
            var src = this.image;
            this.image = document.createElement('img');
            this.image.src = src;
        }
    },

    render: function (ctx) {
        if (this.image) {
            ctx.drawImage(this.image, this.getX(), this.getY());
        }
        return ctx;
    }
};

Alchemy.v.CvsElement = Alchemy.brew({
    extend: Alchemy.v.BasicElement
}, Alchemy.v.CvsElement);
