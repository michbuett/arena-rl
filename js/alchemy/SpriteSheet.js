(function () {
    'use strict';

    var alchemy = require('./alchemy.js');

    /**
     * Description
     */
    alchemy.formula.add({
        name: 'arena.alchemy.SpriteSheet',
        alias: 'SpriteSheet',

        overrides: {

            /**
             * @cfg {Number} spriteWidth
             * The width of a single sprite
             */
            spriteWidth: 0,

            /**
             * @cfg {Number} spriteHeight
             * The height of a single sprite
             */
            spriteHeight: 0,

            /**
             * @cfg {Image/Canvas} image
             * the initial sprite sheet image
             */
            image: undefined,

            /**
             * @property width
             * @type Number
             * @private
             */
            width: 0,

            /**
             * @property sprites
             * @type Number
             * @private
             */
            height: 0,

            /**
             * @property sprites
             * @type Array
             * @private
             */
            sprites: undefined,

            init: function () {
                if (this.image) {
                    this.extractSprites(this.image);
                }
            },

            /**
             * splits the initial sheet image and extracts the sprites
             * @private
             */
            extractSprites: function (image) {
                var x = 0,
                    y = 0,
                    sw = this.spriteWidth,
                    sh = this.spriteHeight,
                    spriteCvs,
                    spriteCtx;

                this.width = image.width;
                this.height = image.height;
                this.sprites = [];

                while (y + sh <= this.height) {
                    x = 0;
                    while (x + sw <= this.width) {
                        spriteCvs = document.createElement('canvas');
                        spriteCvs.width = sw;
                        spriteCvs.height = sh;
                        spriteCtx = spriteCvs.getContext('2d');
                        spriteCtx.drawImage(image, x, y, sw, sh, 0, 0, sw, sh);
                        this.sprites.push(spriteCvs);
                        x += sw;
                    }
                    y += sh;
                }
            },

            /**
             * @param {Number} index
             *      the index of the sprite to get
             *
             * @return {Object}
             */
            getSprite: function (index) {
                return this.sprites && this.sprites[index];
            }
        }
    });
}());
