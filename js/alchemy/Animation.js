(function () {
    var Animation = {

        /**
         * the animation frames; each frame oject provides the following properties:
         * <pre><code>
         * {
         *     image: {CanvasElement}, // the actual frame image
         *     durration: {Number}
         * }
         * </code></pre>
         *
         * @property frames
         * @type Array
         * @private
         */
        frames: undefined,

        /**
         * @property currentFrame
         * @type Number
         * @private
         */
        currentFrame: undefined,

        /**
         * @property currentIteration
         * @type Number
         * @private
         */
        currentIteration: undefined,

        /**
         * @property iterations
         * @type Number
         */
        iterations: -1,

        constructor: function (cfg) {
            _super.call(this, cfg);

            var sheet = Alchemy.resources.get(this.src);
            var framesCfg = this.frames;

            this.frames = [];
            for (var i = 0; i < framesCfg.length; i++) {
                var fCfg = framesCfg[i];
                if (Alchemy.isNumber(fCfg)) {
                    fCfg = {
                        index: fCfg
                    };
                }
                if (this.defaults) {
                    Alchemy.mix(fCfg, this.defaults, {
                        override: false
                    });
                }
                this.frames[i] = Alchemy.mix(fCfg, {
                    image: sheet.getSprite(fCfg.index)
                });
            }
        },

        start: function () {
            this.currentIteration = 0;
            this.setCurrentFrame(0);
        },

        stop: function () {
            this.currentIteration = null;
        },

        isPlaying: function () {
            return Alchemy.isNumber(this.currentIteration);
        },

        nextFrame: function () {
            if (this.currentFrame < this.frames.length - 1) {
                this.setCurrentFrame(this.currentFrame + 1);
            } else if (this.iterations < 0 || this.currentIteration < this.iterations) {
                this.setCurrentFrame(0);
                this.currentIteration++;
            }
        },

        setCurrentFrame: function (frameIdx) {
            this.currentFrame = frameIdx;
            this.image = this.frames[this.currentFrame].image;
            this.frameStartTime = Date.now();
            this.fireEvent('framechanged');
        },

        update: function () {
            if (this.isPlaying()) {
                var cFrame = this.frames[this.currentFrame];
                if (Date.now() - this.frameStartTime > cFrame.durration) {
                    this.nextFrame();
                }
            }
        }
    };

    Alchemy.v.Animation = Alchemy.brew({
        extend: Alchemy.v.CvsElement
    }, Animation);
})();
