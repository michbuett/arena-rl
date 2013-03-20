(function () {
    var AnimatedEl = {

        animations: undefined,

        constructor: function (cfg) {
            var animations = cfg && cfg.animations,
                animKey,
                animCfg;

            _super.call(this, cfg);

            this.animations = {};

            if (animations) {
                for (animKey in animations) {
                    if (animations.hasOwnProperty(animKey)) {
                        animCfg = animations[animKey];

                        if (Alchemy.isArray(animCfg)) {
                            animCfg = {
                                frames: animCfg
                            };
                        }
                        this.addAnimation(animKey, animCfg);
                    }
                }
            }
        },

        addAnimation: function (key, cfg) {
            if (this.defaults) {
                cfg = Alchemy.mix(cfg, this.defaults, {
                    override: false
                });
            }
            cfg.x = cfg.x || 0;
            cfg.y = cfg.y || 0;
            cfg.listeners = {
                framechanged: {
                    fn: this.clear,
                    scope: this
                }
            };

            this.animations[key] = Alchemy.v.Animation.create(cfg);
        },

        play: function (anim) {
            if (this.animations[anim]) {
                this.currAnim = anim;
                this.animations[anim].start();
            }
        },

        getCurrentAnimation: function () {
            return this.animations[this.currAnim];
        },

        clear: function () {
            var renderCtx = this.getContext();
            if (renderCtx) {
                renderCtx.clearRect(0, 0, this.getWidth(), this.getHeight());
            }
        },

        renderCvsContent: function () {
            var renderCtx = this.getContext(),
                anim = this.getCurrentAnimation();
            if (anim && renderCtx) {
                anim.render(renderCtx);
            }
        },

        update: function () {
            var anim = this.getCurrentAnimation();
            if (anim) {
                anim.update();
            }
        }
    };

    Alchemy.v.AnimatedEl = Alchemy.brew({
        extend: Alchemy.v.CvsContainer
    }, AnimatedEl);
})();
